use std::collections::HashMap;

use crate::builtin_types::*;
use crate::parameter::*;

use std::collections::BTreeSet;

use ruffbox_synth::ruffbox::synth::SynthParameter;

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;
use std::sync;

pub fn load_part(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let mut gens = Vec::new();
    let mut proxies = Vec::new();

    let name: String = if let Some(EvaluatedExpr::String(s)) = tail_drain.next() {
        s
    } else {
        return None;
    };

    for c in tail_drain {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::Generator(g)) => gens.push(g),
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(mut gl)) => gens.append(&mut gl),
            EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(p)) => proxies.push(p),
            EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(mut pl)) => proxies.append(&mut pl),
            _ => {}
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::LoadPart(
        (name, Part::Combined(gens, proxies)),
    ))))
}

pub fn freeze_buffer(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let freeze_buffer = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
        f as usize
    } else {
        1
    };

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::FreezeBuffer(freeze_buffer),
    )))
}

pub fn load_sample(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut collect_keywords = false;

    let mut keywords: Vec<String> = Vec::new();
    let mut path: String = "".to_string();
    let mut set: String = "".to_string();

    while let Some(c) = tail_drain.next() {
        if collect_keywords {
            if let EvaluatedExpr::Symbol(ref s) = c {
                keywords.push(s.to_string());
                continue;
            } else {
                collect_keywords = false;
            }
        }

        if let EvaluatedExpr::Keyword(k) = c {
            match k.as_str() {
                "keywords" => {
                    collect_keywords = true;
                    continue;
                }
                "set" => {
                    if let EvaluatedExpr::Symbol(n) = tail_drain.next().unwrap() {
                        set = n.to_string();
                    }
                }
                "path" => {
                    if let EvaluatedExpr::String(n) = tail_drain.next().unwrap() {
                        path = n.to_string();
                    }
                }
                _ => println!("{}", k),
            }
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::LoadSample((set, keywords, path)),
    )))
}

pub fn load_sample_sets(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let path = if let EvaluatedExpr::String(n) = tail_drain.next().unwrap() {
        n
    } else {
        "".to_string()
    };

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::LoadSampleSets(path),
    )))
}

pub fn load_sample_set(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let path = if let EvaluatedExpr::String(n) = tail_drain.next().unwrap() {
        n
    } else {
        "".to_string()
    };

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::LoadSampleSet(path),
    )))
}

pub fn tmod(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::Tmod(
        match tail_drain.next() {
            Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => p,
            Some(EvaluatedExpr::Float(f)) => Parameter::with_value(f),
            _ => Parameter::with_value(1.0),
        },
    ))))
}

pub fn latency(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::Latency(
        match tail_drain.next() {
            Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => p,
            Some(EvaluatedExpr::Float(f)) => Parameter::with_value(f),
            _ => Parameter::with_value(0.05),
        },
    ))))
}

pub fn bpm(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::Bpm(
        match tail_drain.next() {
            Some(EvaluatedExpr::Float(f)) => 60000.0 / f,
            _ => 200.0,
        },
    ))))
}

pub fn default_duration(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::DefaultDuration(match tail_drain.next() {
            Some(EvaluatedExpr::Float(f)) => f,
            _ => 200.0,
        }),
    )))
}

pub fn globres(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::GlobRes(
        match tail_drain.next() {
            Some(EvaluatedExpr::Float(f)) => f,
            _ => 400000.0,
        },
    ))))
}

pub fn reverb(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let mut param_map = HashMap::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "damp" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        param_map.insert(SynthParameter::ReverbDampening, f);
                    }
                }
                "mix" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        param_map.insert(SynthParameter::ReverbMix, f.clamp(0.01, 0.99));
                    }
                }
                "roomsize" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        param_map.insert(SynthParameter::ReverbRoomsize, f.clamp(0.01, 0.99));
                    }
                }

                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::GlobalRuffboxParams(param_map),
    )))
}

pub fn delay(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let mut param_map = HashMap::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "damp-freq" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        param_map.insert(
                            SynthParameter::DelayDampeningFrequency,
                            f.clamp(20.0, 18000.0),
                        );
                    }
                }
                "feedback" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        param_map.insert(SynthParameter::DelayFeedback, f.clamp(0.01, 0.99));
                    }
                }
                "mix" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        param_map.insert(SynthParameter::DelayMix, f.clamp(0.01, 0.99));
                    }
                }
                "time" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        param_map.insert(SynthParameter::DelayTime, (f / 1000.0).clamp(0.01, 1.99));
                    }
                }

                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::GlobalRuffboxParams(param_map),
    )))
}

pub fn export_dot(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    // filename
    let filename = if let Some(EvaluatedExpr::String(s)) = tail_drain.next() {
        s
    } else {
        return None;
    };

    match tail_drain.next() {
        Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(g))) => Some(EvaluatedExpr::BuiltIn(
            BuiltIn::Command(Command::ExportDotStatic((filename, g))),
        )),
        Some(EvaluatedExpr::Keyword(k)) => {
            match k.as_str() {
                "part" => {
                    if let Some(EvaluatedExpr::Symbol(part_name)) = tail_drain.next() {
                        // collect next symbols
                        Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
                            Command::ExportDotPart((filename, part_name)),
                        )))
                    } else {
                        None
                    }
                }
                "live" => {
                    let mut id_tags = BTreeSet::new();
                    // expect more tags
                    while let Some(EvaluatedExpr::Symbol(si)) = tail_drain.next() {
                        id_tags.insert(si);
                    }
                    // collect next symbols
                    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
                        Command::ExportDotRunning((filename, id_tags)),
                    )))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn once(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);
    let mut sound_events = Vec::new();
    let mut control_events = Vec::new();

    for c in tail_drain {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(mut e)) => sound_events.push(e.get_static()),
            EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(c)) => control_events.push(c),
            _ => {}
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::Once((
        sound_events,
        control_events,
    )))))
}

pub fn step_part(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
        Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::StepPart(
            s,
        ))))
    } else {
        None
    }
}

pub fn clear(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::Clear)))
}

pub fn connect_visualizer(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(Command::ConnectVisualizer)))
}
