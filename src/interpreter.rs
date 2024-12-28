use rosc::OscType;

use std::sync;
use std::thread;

use crate::builtin_types::*;

use crate::commands;
use crate::eval::EvaluatedExpr;
use crate::file_interpreter;
use crate::midi_input;
use crate::osc_receiver::OscReceiver;

use crate::session::Session;
use crate::visualizer_client::VisualizerClient;

pub fn interpret_command<const BUFSIZE: usize, const NCHAN: usize>(
    c: Command,
    session: &Session<BUFSIZE, NCHAN>,
    base_dir: String,
) {
    match c {
        Command::LoadFile(f) => {
            file_interpreter::parse_file(f, session, base_dir);
        }
        Command::Push(id, te) => {
            commands::push(id, te, &session.globals);
        }
        Command::Insert(id, key, value) => {
            commands::insert(id, key, value, &session.globals);
        }
        Command::Print(te) => {
            println!("{te:#?}");
        }
        Command::Clear => {
            let session2 = session.clone();
            thread::spawn(move || {
                Session::clear_session(session2);
                println!("a command (stop session)");
            });
        }
        Command::ConnectVisualizer(excludes) => {
            if !session
                .osc_client
                .vis_connected
                .load(sync::atomic::Ordering::SeqCst)
            {
                let mut wr = session.osc_client.vis.write();
                if wr.is_none() {
                    wr.replace(VisualizerClient::start(excludes));
                }
                session
                    .osc_client
                    .vis_connected
                    .store(true, sync::atomic::Ordering::SeqCst);
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
            let ruffbox2 = sync::Arc::clone(&session.ruffbox);
            let fmap2 = sync::Arc::clone(&session.functions);
            let session2 = session.clone();
            thread::spawn(move || {
                commands::fetch_sample_set(
                    &fmap2,
                    &ruffbox2,
                    session2.sample_set,
                    base_dir,
                    resource,
                );
            });
        }
        Command::LoadSample(set, mut keywords, path, downmix_stereo) => {
            let ruffbox2 = sync::Arc::clone(&session.ruffbox);
            let fmap2 = sync::Arc::clone(&session.functions);
            let session2 = session.clone();
            thread::spawn(move || {
                commands::load_sample(
                    &fmap2,
                    &ruffbox2,
                    session2.sample_set,
                    set,
                    &mut keywords,
                    path,
                    downmix_stereo,
                );
                println!("a command (load sample)");
            });
        }
        Command::LoadSampleAsWavematrix(key, path, method, matrix_size, start) => {
            let session2 = session.clone();
            thread::spawn(move || {
                commands::load_sample_as_wavematrix(
                    session2.sample_set,
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
            let ruffbox2 = sync::Arc::clone(&session.ruffbox);
            let fmap2 = sync::Arc::clone(&session.functions);
            let session2 = session.clone();
            thread::spawn(move || {
                commands::load_sample_sets(
                    &fmap2,
                    &ruffbox2,
                    session2.sample_set,
                    path,
                    downmix_stereo,
                );
                println!("a command (load sample sets)");
            });
        }
        Command::LoadSampleSet(path, downmix_stereo) => {
            let ruffbox2 = sync::Arc::clone(&session.ruffbox);
            let fmap2 = sync::Arc::clone(&session.functions);
            let session2 = session.clone();
            thread::spawn(move || {
                commands::load_sample_set_string(
                    &fmap2,
                    &ruffbox2,
                    session2.sample_set,
                    path,
                    downmix_stereo,
                );
                println!("a command (load sample sets)");
            });
        }
        Command::FreezeBuffer(freezbuf, inbuf) => {
            commands::freeze_buffer(&session.ruffbox, freezbuf, inbuf);
            println!("freeze buffer {inbuf} --> {freezbuf}");
        }
        Command::FreezeAddBuffer(freezbuf, inbuf) => {
            commands::freeze_add_buffer(&session.ruffbox, freezbuf, inbuf);
            println!("freeze-add buffer {inbuf} --> {freezbuf} ");
        }
        Command::Tmod(p) => {
            commands::set_global_tmod(&session.globals, p);
        }
        Command::Latency(p) => {
            commands::set_global_latency(&session.globals, p);
        }
        Command::DefaultDuration(d) => {
            commands::set_default_duration(&session.globals, d);
        }
        Command::Bpm(b) => {
            commands::set_default_duration(&session.globals, b);
        }
        Command::GlobRes(v) => {
            commands::set_global_lifemodel_resources(&session.globals, v);
        }
        Command::GlobalRuffboxParams(mut m) => {
            commands::set_global_ruffbox_parameters(&session.ruffbox, &session.globals, &mut m);
        }
        Command::ExportDotStatic(f, g) => {
            commands::export_dot_static(&f, &g);
        }
        Command::ExportDotRunning((f, t)) => {
            commands::export_dot_running(&f, &t, session);
        }
        Command::Once(mut s, c) => {
            commands::once(session, &mut s, &c);
        }
        Command::StepPart(name) => {
            commands::step_part(session, name);
        }
        Command::OscDefineClient(client_name, host) => {
            commands::define_osc_client(
                client_name,
                host,
                "127.0.0.1:51580".to_string(),
                &session.osc_client.custom,
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
            if let Some(thing) = &session.osc_client.custom.get(&client_name) {
                let _ = thing.value().send_message(osc_addr, osc_args);
            }
            //println!("send msg {client_name} {osc_addr}");
        }
        Command::OscStartReceiver(target) => {
            OscReceiver::start_receiver_thread_udp(target, session.clone(), base_dir);
        }
        Command::MidiStartReceiver(midi_in_port) => {
            let session2 = session.clone();
            thread::spawn(move || {
                midi_input::open_midi_input_port(midi_in_port, session2, base_dir);
            });
        }
        Command::MidiListPorts => {
            midi_input::list_midi_input_ports();
        }
    };
}

pub fn interpret<const BUFSIZE: usize, const NCHAN: usize>(
    parsed_in: EvaluatedExpr,
    session: Session<BUFSIZE, NCHAN>,
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
            Session::handle_context(&mut s, &session);
        }
        EvaluatedExpr::Command(c) => {
            interpret_command(c, &session, base_dir);
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
            println!("an f32 number: {f}")
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(f))) => {
            println!("an f64 number: {f}")
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::UInt128(f))) => {
            println!("a u128 number: {f}")
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Int32(f))) => {
            println!("a i32 number: {f}")
        }
        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Int64(f))) => {
            println!("a i64 number: {f}")
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
            session.functions.usr_lib.insert(name, (pos_args, body));
        }
        EvaluatedExpr::VariableDefinition(name, var, keep_state) => {
            println!("a variable definition {name:#?} keep state {keep_state}");
            // if we keep the state, we check whether we have something under that name
            // (currently a generator or generator list) ... otherwise we overwrite it
            if keep_state {
                match session.globals.remove(&name) {
                    Some((_, TypedEntity::Generator(old))) => match var {
                        TypedEntity::Generator(mut new) => {
                            if old.id_tags == new.id_tags {
                                new.transfer_state(&old);
                            }
                            println!("kept state for {name:?}");
                            session.globals.insert(name, TypedEntity::Generator(new));
                        }
                        TypedEntity::GeneratorList(mut new) => {
                            for gi in new.iter_mut() {
                                if gi.id_tags == old.id_tags {
                                    gi.transfer_state(&old);
                                }
                            }
                            session
                                .globals
                                .insert(name, TypedEntity::GeneratorList(new));
                        }
                        _ => {
                            println!("state incompatible, overwriting");
                            session.globals.insert(name, var);
                        }
                    },
                    // n-to-n comparison, under the assumption of small lists
                    Some((_, TypedEntity::GeneratorList(old))) => match var {
                        TypedEntity::GeneratorList(mut new) => {
                            for go in old {
                                for gi in new.iter_mut() {
                                    if go.id_tags == gi.id_tags {
                                        gi.transfer_state(&go);
                                    }
                                }
                            }
                            println!("kept state for {name:?}");
                            session
                                .globals
                                .insert(name, TypedEntity::GeneratorList(new));
                        }
                        TypedEntity::Generator(mut new) => {
                            for go in old {
                                if go.id_tags == new.id_tags {
                                    new.transfer_state(&go);
                                }
                            }
                            println!("kept state for {name:?}");
                            session.globals.insert(name, TypedEntity::Generator(new));
                        }
                        _ => {
                            println!("state incompatible, overwriting");
                            session.globals.insert(name, var);
                        }
                    },
                    _ => {
                        println!("state incompatible, overwriting");
                        session.globals.insert(name, var);
                    }
                }
            } else {
                session.globals.insert(name, var);
            }
        }
        EvaluatedExpr::Progn(exprs) => {
            for expr in exprs {
                interpret(expr, session.clone(), base_dir.clone());
            }
        }
        EvaluatedExpr::Comparator(c) => {
            println!("a comparator {c:?}");
        }
        _ => println!("unknown"),
    }
}
