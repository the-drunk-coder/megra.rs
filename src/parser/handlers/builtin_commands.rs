use std::collections::HashMap;

use crate::builtin_types::*;
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use std::collections::BTreeSet;

use ruffbox_synth::ruffbox::synth::SynthParameter;

fn handle_load_part(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut gens = Vec::new();
    let mut proxies = Vec::new();

    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::Generator(g) => gens.push(g),
            Atom::GeneratorList(mut gl) => gens.append(&mut gl),
            Atom::PartProxy(p) => proxies.push(p),
            Atom::ProxyList(mut pl) => proxies.append(&mut pl),
            _ => {}
        }
    }

    Atom::Command(Command::LoadPart((name, Part::Combined(gens, proxies))))
}

fn handle_freeze_buffer(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    let freeze_buffer = if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
        f as usize
    } else {
        1
    };

    Atom::Command(Command::FreezeBuffer(freeze_buffer))
}

fn handle_load_sample(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    let mut collect_keywords = false;

    let mut keywords: Vec<String> = Vec::new();
    let mut path: String = "".to_string();
    let mut set: String = "".to_string();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_keywords {
            if let Atom::Symbol(ref s) = c {
                keywords.push(s.to_string());
                continue;
            } else {
                collect_keywords = false;
            }
        }

        if let Atom::Keyword(k) = c {
            match k.as_str() {
                "keywords" => {
                    collect_keywords = true;
                    continue;
                }
                "set" => {
                    if let Expr::Constant(Atom::Symbol(n)) = tail_drain.next().unwrap() {
                        set = n.to_string();
                    }
                }
                "path" => {
                    if let Expr::Constant(Atom::Description(n)) = tail_drain.next().unwrap() {
                        path = n.to_string();
                    }
                }
                _ => println!("{}", k),
            }
        }
    }

    Atom::Command(Command::LoadSample((set, keywords, path)))
}

fn handle_load_sample_sets(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let path = if let Expr::Constant(Atom::Description(n)) = tail_drain.next().unwrap() {
        n
    } else {
        "".to_string()
    };

    Atom::Command(Command::LoadSampleSets(path))
}

fn handle_load_sample_set(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let path = if let Expr::Constant(Atom::Description(n)) = tail_drain.next().unwrap() {
        n
    } else {
        "".to_string()
    };

    Atom::Command(Command::LoadSampleSet(path))
}

fn handle_tmod(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    Atom::Command(Command::Tmod(match tail_drain.next() {
        Some(Expr::Constant(Atom::Parameter(p))) => p,
        Some(Expr::Constant(Atom::Float(f))) => Parameter::with_value(f),
        _ => Parameter::with_value(1.0),
    }))
}

fn handle_latency(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    Atom::Command(Command::Latency(match tail_drain.next() {
        Some(Expr::Constant(Atom::Parameter(p))) => p,
        Some(Expr::Constant(Atom::Float(f))) => Parameter::with_value(f),
        _ => Parameter::with_value(0.05),
    }))
}

fn handle_bpm(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    Atom::Command(Command::Bpm(match tail_drain.next() {
        Some(Expr::Constant(Atom::Float(f))) => 60000.0 / f,
        _ => 200.0,
    }))
}

fn handle_default_duration(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    Atom::Command(Command::DefaultDuration(match tail_drain.next() {
        Some(Expr::Constant(Atom::Float(f))) => f,
        _ => 200.0,
    }))
}

fn handle_globres(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    Atom::Command(Command::GlobRes(match tail_drain.next() {
        Some(Expr::Constant(Atom::Float(f))) => f,
        _ => 400000.0,
    }))
}

fn handle_reverb(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut param_map = HashMap::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::Keyword(k) => match k.as_str() {
                "damp" => {
                    if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
                        param_map.insert(SynthParameter::ReverbDampening, f);
                    }
                }
                "mix" => {
                    if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
                        param_map.insert(SynthParameter::ReverbMix, f.clamp(0.01, 0.99));
                    }
                }
                "roomsize" => {
                    if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
                        param_map.insert(SynthParameter::ReverbRoomsize, f.clamp(0.01, 0.99));
                    }
                }

                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    Atom::Command(Command::GlobalRuffboxParams(param_map))
}

fn handle_delay(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut param_map = HashMap::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::Keyword(k) => match k.as_str() {
                "damp-freq" => {
                    if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
                        param_map.insert(
                            SynthParameter::DelayDampeningFrequency,
                            f.clamp(20.0, 18000.0),
                        );
                    }
                }
                "feedback" => {
                    if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
                        param_map.insert(SynthParameter::DelayFeedback, f.clamp(0.01, 0.99));
                    }
                }
                "mix" => {
                    if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
                        param_map.insert(SynthParameter::DelayMix, f.clamp(0.01, 0.99));
                    }
                }
                "time" => {
                    if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
                        param_map.insert(SynthParameter::DelayTime, (f / 1000.0).clamp(0.01, 1.99));
                    }
                }

                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    Atom::Command(Command::GlobalRuffboxParams(param_map))
}

fn handle_export_dot(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    // filename
    let filename = if let Some(Expr::Constant(Atom::Description(s))) = tail_drain.next() {
        s
    } else {
        return Atom::Nothing;
    };

    match tail_drain.next() {
        Some(Expr::Constant(Atom::Generator(g))) => {
            Atom::Command(Command::ExportDotStatic((filename, g)))
        }
        Some(Expr::Constant(Atom::Keyword(k))) => {
            match k.as_str() {
                "part" => {
                    if let Some(Expr::Constant(Atom::Symbol(part_name))) = tail_drain.next() {
                        // collect next symbols
                        Atom::Command(Command::ExportDotPart((filename, part_name)))
                    } else {
                        Atom::Nothing
                    }
                }
                "live" => {
                    let mut id_tags = BTreeSet::new();
                    // expect more tags
                    while let Some(Expr::Constant(Atom::Symbol(si))) = tail_drain.next() {
                        id_tags.insert(si);
                    }
                    // collect next symbols
                    Atom::Command(Command::ExportDotRunning((filename, id_tags)))
                }
                _ => Atom::Nothing,
            }
        }
        _ => Atom::Nothing,
    }
}

fn handle_once(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut sound_events = Vec::new();
    let mut control_events = Vec::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::SoundEvent(mut e) => sound_events.push(e.get_static()),
            Atom::ControlEvent(c) => control_events.push(c),
            _ => {}
        }
    }

    Atom::Command(Command::Once((sound_events, control_events)))
}

fn handle_step_part(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    if let Some(Expr::Constant(Atom::Symbol(s))) = tail_drain.next() {
        Atom::Command(Command::StepPart(s))
    } else {
        Atom::Nothing
    }
}

pub fn handle(cmd: BuiltInCommand, tail: &mut Vec<Expr>) -> Atom {
    match cmd {
        BuiltInCommand::Clear => Atom::Command(Command::Clear),
        BuiltInCommand::Tmod => handle_tmod(tail),
        BuiltInCommand::Latency => handle_latency(tail),
        BuiltInCommand::Reverb => handle_reverb(tail),
        BuiltInCommand::Delay => handle_delay(tail),
        BuiltInCommand::GlobRes => handle_globres(tail),
        BuiltInCommand::LoadSample => handle_load_sample(tail),
        BuiltInCommand::LoadSampleSet => handle_load_sample_set(tail),
        BuiltInCommand::LoadSampleSets => handle_load_sample_sets(tail),
        BuiltInCommand::LoadPart => handle_load_part(tail),
        BuiltInCommand::StepPart => handle_step_part(tail),
        BuiltInCommand::ExportDot => handle_export_dot(tail),
        BuiltInCommand::Once => handle_once(tail),
        BuiltInCommand::FreezeBuffer => handle_freeze_buffer(tail),
        BuiltInCommand::DefaultDuration => handle_default_duration(tail),
        BuiltInCommand::Bpm => handle_bpm(tail),
    }
}
