use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use ruffbox_synth::ruffbox::synth::SynthParameterLabel;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

pub fn nuc(
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
    let mut keep_root = false;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => ev_vec.push(SourceEvent::Sound(e)),
            EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(c)) => {
                ev_vec.push(SourceEvent::Control(c))
            }
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Float(n)) => {
                        dur = Parameter::with_value(n);
                    }
                    Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "keep" => {
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        keep_root = b;
                    }
                }
                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    // only re-generate if necessary ...
    let pfa = if !keep_root {
        event_mapping.insert('a', ev_vec);

        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev
            .params
            .insert(SynthParameterLabel::Duration, Box::new(dur.clone()));
        duration_mapping.insert(('a', 'a'), dur_ev);
        // one rule to rule them all
        rules.push(Rule {
            source: vec!['a'],
            symbol: 'a',
            probability: 1.0,
        });

        Pfa::<char>::infer_from_rules(&mut rules, false)
    } else {
        Pfa::<char>::new()
    };

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            duration_mapping,
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        keep_root,
    })))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parser::*;

    #[test]
    fn test_eval_nuc() {
        let snippet = "(nuc 'da (bd))";
        let mut functions = FunctionMap::new();
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        functions
            .fmap
            .insert("nuc".to_string(), eval::constructors::nuc::nuc);
        functions.fmap.insert("bd".to_string(), |_, _, _, _, _| {
            Some(EvaluatedExpr::String("bd".to_string()))
        });

        let globals = sync::Arc::new(GlobalParameters::new());

        match eval_from_str(
            snippet,
            &functions,
            &globals,
            &sample_set,
            OutputMode::Stereo,
        ) {
            Ok(res) => {
                assert!(matches!(res, EvaluatedExpr::BuiltIn(BuiltIn::Generator(_))));
            }
            Err(e) => {
                println!("err {}", e);
                assert!(false)
            }
        }
    }
}
