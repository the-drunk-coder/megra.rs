use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use vom_rs::pfa::{Pfa, Rule};

pub fn construct_nucleus(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
        n
    } else {
        "".to_string()
    };

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();
    let mut rules = Vec::new();

    let mut dur: Option<Parameter> = Some(Parameter::with_value(200.0));
    let mut ev_vec = Vec::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::SoundEvent(e) => ev_vec.push(SourceEvent::Sound(e)),
            Atom::ControlEvent(c) => ev_vec.push(SourceEvent::Control(c)),
            Atom::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(Expr::Constant(Atom::Float(n))) => {
                        dur = Some(Parameter::with_value(n));
                    }
                    Some(Expr::Constant(Atom::Parameter(p))) => {
                        dur = Some(p);
                    }
                    _ => {}
                },
                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    event_mapping.insert('a', ev_vec);

    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev
        .params
        .insert(SynthParameter::Duration, Box::new(dur.unwrap()));
    duration_mapping.insert(('a', 'a'), dur_ev);
    // one rule to rule them all
    rules.push(Rule {
        source: vec!['a'],
        symbol: 'a',
        probability: 1.0,
    });

    let pfa = Pfa::<char>::infer_from_rules(&mut rules);
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
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}
