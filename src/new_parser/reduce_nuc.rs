use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::new_parser::{BuiltIn2, EvaluatedExpr};

pub fn reduce_nuc(
    tail: &mut Vec<EvaluatedExpr>,
    global_parameters: &sync::Arc<GlobalParameters>,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    // name is the first symbol
    let name = if let Some(EvaluatedExpr::String(n)) = tail_drain.next() {
        n
    } else {
        "".to_string()
    };

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();
    let mut rules = Vec::new();

    let mut dur: Parameter = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        Parameter::with_value(*d)
    } else {
        unreachable!()
    };

    let mut ev_vec = Vec::new();

    /*
    while let Some(EvaluatedExpr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::SoundEvent(e) => ev_vec.push(SourceEvent::Sound(e)),
            Atom::ControlEvent(c) => ev_vec.push(SourceEvent::Control(c)),
            Atom::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(Expr::Constant(Atom::Float(n))) => {
                        dur = Parameter::with_value(n);
                    }
                    Some(Expr::Constant(Atom::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }*/

    event_mapping.insert('a', ev_vec);

    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev
        .params
        .insert(SynthParameter::Duration, Box::new(dur.clone()));
    duration_mapping.insert(('a', 'a'), dur_ev);
    // one rule to rule them all
    rules.push(Rule {
        source: vec!['a'],
        symbol: 'a',
        probability: 1.0,
    });

    let pfa = Pfa::<char>::infer_from_rules(&mut rules, false);
    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Some(EvaluatedExpr::BuiltIn(BuiltIn2::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })))
}
