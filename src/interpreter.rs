use parking_lot::Mutex;
use rosc::OscType;

use std::sync;
use std::thread;

use ruffbox_synth::ruffbox::RuffboxControls;

use crate::builtin_types::*;

use crate::commands;
use crate::midi_input;
use crate::osc_receiver::OscReceiver;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::{OutputMode, Session};
use crate::visualizer_client::VisualizerClient;

#[allow(clippy::too_many_arguments)]
pub fn interpret_command<const BUFSIZE: usize, const NCHAN: usize>(
    c: Command,
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    globals: &sync::Arc<GlobalVariables>,
    output_mode: OutputMode,
    base_dir: String,
) {
    match c {
        Command::Push(id, te) => {
            commands::push(id, te, globals);
        }
        Command::Insert(id, key, value) => {
            commands::insert(id, key, value, globals);
        }
        Command::Print(te) => {
            println!("{te:#?}");
        }
        Command::Clear => {
            let session2 = sync::Arc::clone(session);
            thread::spawn(move || {
                Session::clear_session(&session2);
                println!("a command (stop session)");
            });
        }
        Command::ConnectVisualizer => {
            let mut session = session.lock();
            if session.osc_client.vis.is_none() {
                session.osc_client.vis = Some(sync::Arc::new(VisualizerClient::start()));
            } else {
                println!("visualizer already connected !");
            }
        }
        Command::StartRecording(prefix, rec_input) => {
            commands::start_recording(session, prefix, base_dir, rec_input);
        }
        Command::StopRecording => {
            commands::stop_recording(session);
        }
        Command::ImportSampleSet(resource) => {
            let ruffbox2 = sync::Arc::clone(ruffbox);
            let fmap2 = sync::Arc::clone(function_map);
            let sample_set2 = sync::Arc::clone(sample_set);
            thread::spawn(move || {
                commands::fetch_sample_set(&fmap2, &ruffbox2, &sample_set2, base_dir, resource);
            });
        }
        Command::LoadSample(set, mut keywords, path, downmix_stereo) => {
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
                    downmix_stereo,
                );
                println!("a command (load sample)");
            });
        }
        Command::LoadSampleAsWavematrix(key, path, method, matrix_size, start) => {
            let sample_set2 = sync::Arc::clone(sample_set);
            thread::spawn(move || {
                commands::load_sample_as_wavematrix(
                    &sample_set2,
                    key,
                    path,
                    &method,
                    matrix_size,
                    start,
                );
                println!("a command (load wavematrix)");
            });
        }
        Command::LoadSampleSets(path, downmix_stereo) => {
            let ruffbox2 = sync::Arc::clone(ruffbox);
            let fmap2 = sync::Arc::clone(function_map);
            let sample_set2 = sync::Arc::clone(sample_set);
            thread::spawn(move || {
                commands::load_sample_sets(&fmap2, &ruffbox2, &sample_set2, path, downmix_stereo);
                println!("a command (load sample sets)");
            });
        }
        Command::LoadSampleSet(path, downmix_stereo) => {
            let ruffbox2 = sync::Arc::clone(ruffbox);
            let fmap2 = sync::Arc::clone(function_map);
            let sample_set2 = sync::Arc::clone(sample_set);
            thread::spawn(move || {
                commands::load_sample_set_string(
                    &fmap2,
                    &ruffbox2,
                    &sample_set2,
                    path,
                    downmix_stereo,
                );
                println!("a command (load sample sets)");
            });
        }
        Command::FreezeBuffer(freezbuf, inbuf) => {
            commands::freeze_buffer(ruffbox, freezbuf, inbuf);
            println!("freeze buffer");
        }
        Command::Tmod(p) => {
            commands::set_global_tmod(globals, p);
        }
        Command::Latency(p) => {
            commands::set_global_latency(globals, p);
        }
        Command::DefaultDuration(d) => {
            commands::set_default_duration(globals, d);
        }
        Command::Bpm(b) => {
            commands::set_default_duration(globals, b);
        }
        Command::GlobRes(v) => {
            commands::set_global_lifemodel_resources(globals, v);
        }
        Command::GlobalRuffboxParams(mut m) => {
            commands::set_global_ruffbox_parameters(ruffbox, globals, &mut m);
        }
        Command::ExportDotStatic(f, g) => {
            commands::export_dot_static(&f, &g);
        }
        Command::ExportDotRunning((f, t)) => {
            commands::export_dot_running(&f, &t, session);
        }
        Command::Once(mut s, mut c) => {
            commands::once(
                ruffbox,
                globals,
                sample_set,
                session,
                &mut s,
                &mut c,
                output_mode,
            );
        }
        Command::StepPart(name) => {
            commands::step_part(ruffbox, globals, sample_set, session, output_mode, name);
        }
        Command::OscDefineClient(client_name, host) => {
            commands::define_osc_client(
                client_name,
                host,
                "127.0.0.1:51580".to_string(),
                &session.lock().osc_client.custom,
            );
        }
        Command::OscSendMessage(client_name, osc_addr, args) => {
            let mut osc_args = Vec::new();
            for arg in args.iter() {
                match arg {
                    TypedEntity::Comparable(Comparable::Float(n)) => {
                        osc_args.push(OscType::Float(*n))
                    }
                    TypedEntity::Comparable(Comparable::Double(n)) => {
                        osc_args.push(OscType::Double(*n))
                    }
                    TypedEntity::Comparable(Comparable::Int32(n)) => {
                        osc_args.push(OscType::Int(*n))
                    }
                    TypedEntity::Comparable(Comparable::Int64(n)) => {
                        osc_args.push(OscType::Long(*n))
                    }
                    TypedEntity::Comparable(Comparable::String(s)) => {
                        osc_args.push(OscType::String(s.to_string()))
                    }
                    TypedEntity::Comparable(Comparable::Symbol(s)) => {
                        osc_args.push(OscType::String(s.to_string()))
                    }
                    _ => {}
                }
            }
            if let Some(thing) = &session.lock().osc_client.custom.get(&client_name) {
                let _ = thing.value().send_message(osc_addr, osc_args);
            }
            //println!("send msg {client_name} {osc_addr}");
        }
        Command::OscStartReceiver(target) => {
            let ruffbox2 = sync::Arc::clone(ruffbox);
            let fmap2 = sync::Arc::clone(function_map);
            let sample_set2 = sync::Arc::clone(sample_set);
            let session2 = sync::Arc::clone(session);
            let globals2 = sync::Arc::clone(globals);
            OscReceiver::start_receiver_thread_udp(
                target,
                fmap2,
                session2,
                ruffbox2,
                sample_set2,
                globals2,
                output_mode,
                base_dir,
            );
        }
        Command::MidiStartReceiver(midi_in_port) => {
            let function_map_midi = sync::Arc::clone(function_map);
            let session_midi = sync::Arc::clone(session);
            let ruffbox_midi = sync::Arc::clone(ruffbox);
            let sam_midi = sync::Arc::clone(sample_set);
            let var_midi = sync::Arc::clone(globals);
            thread::spawn(move || {
                midi_input::open_midi_input_port(
                    midi_in_port,
                    function_map_midi,
                    session_midi,
                    ruffbox_midi,
                    sam_midi,
                    var_midi,
                    output_mode,
                    base_dir,
                );
            });
        }
        Command::MidiListPorts => {
            midi_input::list_midi_input_ports();
        }
    };
}

#[allow(clippy::too_many_arguments)]
pub fn interpret<const BUFSIZE: usize, const NCHAN: usize>(
    parsed_in: EvaluatedExpr,
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    globals: &sync::Arc<GlobalVariables>,
    output_mode: OutputMode,
    base_dir: String,
) {
    match parsed_in {
        EvaluatedExpr::Typed(TypedEntity::Generator(g)) => {
            print!("a generator called \'");
            for tag in g.id_tags.iter() {
                print!("{tag} ");
            }
            println!("\'");
        }
        EvaluatedExpr::Typed(TypedEntity::Parameter(_)) => {
            println!("a parameter");
        }
        EvaluatedExpr::Typed(TypedEntity::ParameterValue(_)) => {
            println!("a parameter value");
        }
        EvaluatedExpr::Typed(TypedEntity::SoundEvent(_)) => {
            println!("a sound event");
        }
        EvaluatedExpr::Typed(TypedEntity::ControlEvent(_)) => {
            println!("a control event");
        }
        EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorModifierFunction(_),
        )) => {
            println!("a gen mod fun");
        }
        EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorProcessor(_),
        )) => {
            println!("a gen proc");
        }
        EvaluatedExpr::Typed(TypedEntity::GeneratorList(gl)) => {
            println!("a gen list");
            for gen in gl.iter() {
                print!("--- a generator called \'");
                for tag in gen.id_tags.iter() {
                    print!("{tag} ");
                }
                println!("\'");
            }
        }
        EvaluatedExpr::SyncContext(mut s) => {
            println!(
                "\n\n############### a context called \'{}\' ###############",
                s.name
            );
            Session::handle_context(&mut s, session, ruffbox, globals, sample_set, output_mode);
        }
        EvaluatedExpr::Command(c) => {
            interpret_command(
                c,
                function_map,
                session,
                ruffbox,
                sample_set,
                globals,
                output_mode,
                base_dir,
            );
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
            println!("a number: {f}")
        }
        EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(l)) => {
            println!("a lazy arithmetic {l:?}")
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
            println!("a symbol: {s}")
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s))) => {
            println!("a string: {s}")
        }
        EvaluatedExpr::Keyword(k) => {
            println!("a keyword: {k}")
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b))) => {
            println!("a boolean: {b}")
        }
        EvaluatedExpr::Typed(TypedEntity::Map(m)) => {
            println!("a map: {m:?}")
        }
        EvaluatedExpr::Typed(TypedEntity::Vec(v)) => {
            println!("a vec: {v:?}")
        }
        EvaluatedExpr::FunctionDefinition(name, pos_args, body) => {
            println!("a function definition: {name} positional args: {pos_args:?}");
            function_map.lock().usr_lib.insert(name, (pos_args, body));
        }
        EvaluatedExpr::VariableDefinition(name, var) => {
            println!("a variable definition {name:#?}");
            globals.insert(name, var);
        }
        EvaluatedExpr::Progn(exprs) => {
            for expr in exprs {
                interpret(
                    expr,
                    function_map,
                    session,
                    ruffbox,
                    sample_set,
                    globals,
                    output_mode,
                    base_dir.clone(),
                );
            }
        }
        EvaluatedExpr::Match(temp, exprs) => {
            if let EvaluatedExpr::Typed(TypedEntity::Comparable(t1)) = *temp {
                for (comp, expr) in exprs {
                    if let EvaluatedExpr::Typed(TypedEntity::Comparable(t2)) = comp {
                        if t1 == t2 {
                            interpret(
                                expr,
                                function_map,
                                session,
                                ruffbox,
                                sample_set,
                                globals,
                                output_mode,
                                base_dir.clone(),
                            );
                        }
                    }
                }
            }
        }
        _ => println!("unknown"),
    }
}
