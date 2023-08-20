use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::eval::resolver::resolve_globals;

use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;

pub fn linear(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    // eval-time resolve
    // ignore function name in this case
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) =
        tail_drain.next()
    {
        n
    } else {
        "".to_string()
    };

    // collect final events and durations in their position in the list
    let mut ev_vecs = Vec::new();
    let mut dur_vec: Vec<DynVal> = Vec::new();

    let dur: DynVal = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        unreachable!()
    };

    for c in tail_drain {
        match c {
            EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Control(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                *dur_vec.last_mut().unwrap() = DynVal::with_value(f);
            }
            _ => println! {"ignored"},
        }
    }

    // generated ids
    let mut last_char: char = '1';

    let mut event_mapping = BTreeMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();

    // collect cycle rules
    let mut rules = Vec::new();
    let len = ev_vecs.len() - 1;

    for (count, ev) in ev_vecs.drain(..).enumerate() {
        event_mapping.insert(last_char, ev);
        if count < len {
            let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();

            let mut dur_ev = Event::with_name("transition".to_string());
            dur_ev.params.insert(
                SynthParameterLabel::Duration,
                ParameterValue::Scalar(dur_vec[count].clone()),
            );

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

    Some(EvaluatedExpr::Typed(TypedEntity::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            duration_mapping,
            label_mapping: None,
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: 200,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        keep_root: false,
    })))
}
