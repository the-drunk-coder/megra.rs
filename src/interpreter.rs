use parking_lot::Mutex;
use std::sync;
use std::thread;

use ruffbox_synth::ruffbox::RuffboxControls;

use crate::builtin_types::*;
use crate::commands;
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::sample_set::SampleSet;
use crate::session::{OutputMode, Session};
use crate::visualizer_client::VisualizerClient;

#[allow(clippy::too_many_arguments)]
pub fn interpret<const BUFSIZE: usize, const NCHAN: usize>(
    parsed_in: EvaluatedExpr,
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    output_mode: OutputMode,
) {
    match parsed_in {
        EvaluatedExpr::BuiltIn(BuiltIn::Generator(g)) => {
            print!("a generator called \'");
            for tag in g.id_tags.iter() {
                print!("{} ", tag);
            }
            println!("\'");
        }
        EvaluatedExpr::BuiltIn(BuiltIn::Parameter(_)) => {
            println!("a parameter");
        }
        EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(_)) => {
            println!("a sound event");
        }
        EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(_)) => {
            println!("a control event");
        }
        EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorModifierFunction(_),
        )) => {
            println!("a gen mod fun");
        }
        EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorProcessor(_),
        )) => {
            println!("a gen proc");
        }
        EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(gl)) => {
            println!("a gen list");
            for gen in gl.iter() {
                print!("--- a generator called \'");
                for tag in gen.id_tags.iter() {
                    print!("{} ", tag);
                }
                println!("\'");
            }
        }
        EvaluatedExpr::BuiltIn(BuiltIn::SyncContext(mut s)) => {
            println!(
                "\n\n############### a context called \'{}\' ###############",
                s.name
            );
            Session::handle_context(
                &mut s,
                session,
                ruffbox,
                parts_store,
                global_parameters,
                output_mode,
            );
        }
        EvaluatedExpr::BuiltIn(BuiltIn::Command(c)) => {
            match c {
                Command::Clear => {
                    let session2 = sync::Arc::clone(session);
                    let parts_store2 = sync::Arc::clone(parts_store);
                    thread::spawn(move || {
                        Session::clear_session(&session2, &parts_store2);
                        println!("a command (stop session)");
                    });
                }
                Command::ConnectVisualizer => {
                    let mut session = session.lock();
                    if session.visualizer_client.is_none() {
                        session.visualizer_client = Some(sync::Arc::new(VisualizerClient::start()));
                    } else {
                        println!("visualizer already connected !");
                    }
                }
                Command::StartRecording => {
                    commands::start_recording(session);
                }
                Command::StopRecording => {
                    commands::stop_recording(session);
                }
                Command::LoadSample((set, mut keywords, path)) => {
                    let ruffbox2 = sync::Arc::clone(ruffbox);
                    let fmap2 = sync::Arc::clone(function_map);
                    let sample_set2 = sync::Arc::clone(sample_set);
                    thread::spawn(move || {
                        commands::load_sample(
                            &fmap2,
                            &ruffbox2,
                            &sample_set2,
                            set,
                            &mut keywords,
                            path,
                        );
                        println!("a command (load sample)");
                    });
                }
                Command::LoadSampleSets(path) => {
                    let ruffbox2 = sync::Arc::clone(ruffbox);
                    let fmap2 = sync::Arc::clone(function_map);
                    let sample_set2 = sync::Arc::clone(sample_set);
                    thread::spawn(move || {
                        commands::load_sample_sets(&fmap2, &ruffbox2, &sample_set2, path);
                        println!("a command (load sample sets)");
                    });
                }
                Command::LoadSampleSet(path) => {
                    let ruffbox2 = sync::Arc::clone(ruffbox);
                    let fmap2 = sync::Arc::clone(function_map);
                    let sample_set2 = sync::Arc::clone(sample_set);
                    thread::spawn(move || {
                        commands::load_sample_set_string(&fmap2, &ruffbox2, &sample_set2, path);
                        println!("a command (load sample sets)");
                    });
                }
                Command::LoadPart((name, part)) => {
                    commands::load_part(parts_store, name, part);
                    println!("a command (load part)");
                }
                Command::FreezeBuffer(freezbuf, inbuf) => {
                    commands::freeze_buffer(ruffbox, freezbuf, inbuf);
                    println!("freeze buffer");
                }
                Command::Tmod(p) => {
                    commands::set_global_tmod(global_parameters, p);
                }
                Command::Latency(p) => {
                    commands::set_global_latency(global_parameters, p);
                }
                Command::DefaultDuration(d) => {
                    commands::set_default_duration(global_parameters, d);
                }
                Command::Bpm(b) => {
                    commands::set_default_duration(global_parameters, b);
                }
                Command::GlobRes(v) => {
                    commands::set_global_lifemodel_resources(global_parameters, v);
                }
                Command::GlobalRuffboxParams(m) => {
                    commands::set_global_ruffbox_parameters(ruffbox, &m);
                }
                Command::ExportDotStatic((f, g)) => {
                    commands::export_dot_static(&f, &g);
                }
                Command::ExportDotRunning((f, t)) => {
                    commands::export_dot_running(&f, &t, session);
                }
                Command::ExportDotPart((f, p)) => {
                    commands::export_dot_part(&f, &p, parts_store);
                }
                Command::Once((mut s, mut c)) => {
                    commands::once(
                        ruffbox,
                        parts_store,
                        global_parameters,
                        session,
                        &mut s,
                        &mut c,
                        output_mode,
                    );
                }
                Command::StepPart(name) => {
                    commands::step_part(
                        ruffbox,
                        parts_store,
                        global_parameters,
                        session,
                        output_mode,
                        name,
                    );
                }
            };
        }
        EvaluatedExpr::Float(f) => {
            println!("a number: {}", f)
        }
        EvaluatedExpr::Symbol(s) => {
            println!("a symbol: {}", s)
        }
        EvaluatedExpr::String(s) => {
            println!("a string: {}", s)
        }
        EvaluatedExpr::Keyword(k) => {
            println!("a keyword: {}", k)
        }
        EvaluatedExpr::Boolean(b) => {
            println!("a boolean: {}", b)
        }
        _ => println!("unknown"),
    }
}
