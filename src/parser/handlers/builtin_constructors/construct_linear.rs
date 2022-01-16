use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

/// construct a simple linear sequence of events ... no tricks here
pub fn construct_linear(
    tail: &mut Vec<Expr>,
    global_parameters: &sync::Arc<GlobalParameters>,
) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
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
            Expr::Constant(Atom::SoundEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            Expr::Constant(Atom::ControlEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Control(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            Expr::Constant(Atom::Float(f)) => {
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

    Atom::Generator(Generator {
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
    })
}
