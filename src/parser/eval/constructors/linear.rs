use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

pub fn linear(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    global_parameters: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // ignore function name in this case
    tail_drain.next();

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Symbol(n)) = tail_drain.next() {
        n
    } else {
        "".to_string()
    };

    // collect final events and durations in their position in the list
    let mut ev_vecs = Vec::new();
    let mut dur_vec: Vec<Parameter> = Vec::new();

    let dur: Parameter = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        Parameter::with_value(*d)
    } else {
        unreachable!()
    };

    for c in tail_drain {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Control(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            EvaluatedExpr::Float(f) => {
                *dur_vec.last_mut().unwrap() = Parameter::with_value(f);
            }
            _ => println! {"ignored"},
        }
    }

    // generated ids
    let mut last_char: char = '1';

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();

    // collect cycle rules
    let mut rules = Vec::new();
    let len = ev_vecs.len() - 1;

    for (count, ev) in ev_vecs.drain(..).enumerate() {
        event_mapping.insert(last_char, ev);
        if count < len {
            let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();

            let mut dur_ev = Event::with_name("transition".to_string());
            dur_ev
                .params
                .insert(SynthParameter::Duration, Box::new(dur_vec[count].clone()));

            rules.push(Rule {
                source: vec![last_char],
                symbol: next_char,
                probability: 1.0,
            });

            duration_mapping.insert((last_char, next_char), dur_ev);

            last_char = next_char;
        }
    }

    // don't remove orphans here because the first state is technically
    // "orphan"
    let pfa = Pfa::<char>::infer_from_rules(&mut rules, false);

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: 200,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })))
}
