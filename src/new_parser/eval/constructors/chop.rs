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
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

pub fn chop(
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

    // name is the first symbol
    let slices: usize = if let Some(EvaluatedExpr::Float(n)) = tail_drain.next() {
        n as usize
    } else {
	8
    };

    let mut dur: Parameter = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        Parameter::with_value(*d)
    } else {
        unreachable!()
    };

    let mut repetition_chance: f32 = 0.0;
    let mut randomize_chance: f32 = 0.0;
    let mut max_repetitions: f32 = 0.0;

    let mut events = Vec::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn2::SoundEvent(e)) => {
                events.push(e);
            }
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Float(n)) => {
                        dur = Parameter::with_value(n);
                    }
                    Some(EvaluatedExpr::BuiltIn(BuiltIn2::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "rep" => {
                    if let EvaluatedExpr::Float(n) = tail_drain.next().unwrap() {
                        repetition_chance = n;
                    }
                }
                "rnd" => {
                    if let EvaluatedExpr::Float(n) = tail_drain.next().unwrap() {
                        randomize_chance = n;
                    }
                }
                "max-rep" => {
                    if let EvaluatedExpr::Float(n) = tail_drain.next().unwrap() {
                        max_repetitions = n;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    let mut chopped_events = Vec::new();

    for s in 0..slices {
        let mut slice_events = Vec::new();

        for ev in events.iter() {
            let mut slice_event = ev.clone();
            slice_event.params.insert(
                SynthParameter::PlaybackStart,
                Box::new(Parameter::with_value(s as f32 * (1.0 / slices as f32))),
            );

            let sus = if let Some(old_sus) = slice_event.params.get(&SynthParameter::Sustain) {
                old_sus.static_val / slices as f32
            } else {
                dur.clone().static_val
            };

            slice_event.params.insert(
                SynthParameter::Sustain,
                Box::new(Parameter::with_value(sus)),
            );

            slice_events.push(SourceEvent::Sound(slice_event));
        }

        chopped_events.push(slice_events)
    }

    let mut rules = Vec::new();
    let mut last_char: char = '!';
    let first_char = last_char;

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();
    let mut count = 0;
    let num_events = chopped_events.len();

    for ev in chopped_events.drain(..) {
        let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();

        event_mapping.insert(last_char, ev);

        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev
            .params
            .insert(SynthParameter::Duration, Box::new(dur.clone()));
        duration_mapping.insert((last_char, next_char), dur_ev);

        if count < num_events - 1 {
            if repetition_chance > 0.0 {
                //println!("add rep chance");
                // repetition rule
                rules.push(Rule {
                    source: vec![last_char],
                    symbol: last_char,
                    probability: repetition_chance / 100.0,
                });

                // next rule
                rules.push(Rule {
                    source: vec![last_char],
                    symbol: next_char,
                    probability: 1.0 - (repetition_chance / 100.0),
                });

                // endless repetition allowed per default ...
                if max_repetitions >= 2.0 {
                    let mut max_rep_source = Vec::new();
                    for _ in 0..max_repetitions as usize {
                        max_rep_source.push(last_char);
                    }
                    // max repetition rule
                    rules.push(Rule {
                        source: max_rep_source,
                        symbol: next_char,
                        probability: 1.0,
                    });
                }
            } else {
                rules.push(Rule {
                    source: vec![last_char],
                    symbol: next_char,
                    probability: 1.0,
                });
            }

            last_char = next_char;
        }

        count += 1;
    }

    // if our cycle isn't empty ...
    if count != 0 {
        // close the cycle
        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev
            .params
            .insert(SynthParameter::Duration, Box::new(dur.clone()));
        duration_mapping.insert((last_char, first_char), dur_ev);

        rules.push(Rule {
            source: vec![last_char],
            symbol: first_char,
            probability: 1.0,
        });
    }

    let mut pfa = Pfa::<char>::infer_from_rules(&mut rules, true);

    // this seems to be heavy ...
    // what's so heavy here ??
    if randomize_chance > 0.0 {
        //println!("add rnd chance");
        pfa.randomize_edges(randomize_chance, randomize_chance);
        pfa.rebalance();
    }

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
