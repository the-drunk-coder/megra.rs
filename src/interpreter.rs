use parking_lot::Mutex;
use rosc::OscType;
use std::collections::HashMap;
use std::sync;
use std::thread;

use ruffbox_synth::ruffbox::RuffboxControls;

use crate::builtin_types::*;
use crate::commands;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::{OutputMode, Session};
use crate::visualizer_client::VisualizerClient;

#[allow(clippy::too_many_arguments)]
pub fn interpret_command<const BUFSIZE: usize, const NCHAN: usize>(
    c: Command,
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    midi_callback_map: &sync::Arc<Mutex<HashMap<u8, Command>>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    var_store: &sync::Arc<VariableStore>,
    output_mode: OutputMode,
    base_dir: String,
) {
    match c {
        Command::Clear => {
            let session2 = sync::Arc::clone(session);
            let var_store2 = sync::Arc::clone(var_store);
            thread::spawn(move || {
                Session::clear_session(&session2, &var_store2);
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
        Command::LoadPart((name, part)) => {
            commands::load_part(var_store, name, part);
            println!("a command (load part)");
        }
        Command::FreezeBuffer(freezbuf, inbuf) => {
            commands::freeze_buffer(ruffbox, freezbuf, inbuf);
            println!("freeze buffer");
        }
        Command::Tmod(p) => {
            commands::set_global_tmod(var_store, p);
        }
        Command::Latency(p) => {
            commands::set_global_latency(var_store, p);
        }
        Command::DefaultDuration(d) => {
            commands::set_default_duration(var_store, d);
        }
        Command::Bpm(b) => {
            commands::set_default_duration(var_store, b);
        }
        Command::GlobRes(v) => {
            commands::set_global_lifemodel_resources(var_store, v);
        }
        Command::GlobalRuffboxParams(mut m) => {
            commands::set_global_ruffbox_parameters(ruffbox, var_store, &mut m);
        }
        Command::ExportDotStatic((f, g)) => {
            commands::export_dot_static(&f, &g);
        }
        Command::ExportDotRunning((f, t)) => {
            commands::export_dot_running(&f, &t, session);
        }
        Command::ExportDotPart((f, p)) => {
            commands::export_dot_part(&f, p, var_store);
        }
        Command::Once((mut s, mut c)) => {
            commands::once(
                ruffbox,
                var_store,
                sample_set,
                session,
                &mut s,
                &mut c,
                output_mode,
            );
        }
        Command::StepPart(name) => {
            commands::step_part(ruffbox, var_store, sample_set, session, output_mode, name);
        }
        Command::DefineMidiCallback(key, c) => {
            midi_callback_map.lock().insert(key, *c);
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
                    TypedEntity::Float(n) => osc_args.push(OscType::Float(*n)),
                    TypedEntity::String(s) => osc_args.push(OscType::String(s.to_string())),
                    TypedEntity::Symbol(s) => osc_args.push(OscType::String(s.to_string())),
                    _ => {}
                }
            }
            if let Some(thing) = &session.lock().osc_client.custom.get(&client_name) {
                let _ = thing.value().send_message(osc_addr, osc_args);
            }
            //println!("send msg {client_name} {osc_addr}");
        }
    };
}

#[allow(clippy::too_many_arguments)]
pub fn interpret<const BUFSIZE: usize, const NCHAN: usize>(
    parsed_in: EvaluatedExpr,
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    midi_callback_map: &sync::Arc<Mutex<HashMap<u8, Command>>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    var_store: &sync::Arc<VariableStore>,
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
            Session::handle_context(&mut s, session, ruffbox, var_store, sample_set, output_mode);
        }
        EvaluatedExpr::Command(c) => {
            interpret_command(
                c,
                function_map,
                midi_callback_map,
                session,
                ruffbox,
                sample_set,
                var_store,
                output_mode,
                base_dir,
            );
        }
        EvaluatedExpr::Typed(TypedEntity::Float(f)) => {
            println!("a number: {f}")
        }
        EvaluatedExpr::Typed(TypedEntity::Symbol(s)) => {
            println!("a symbol: {s}")
        }
        EvaluatedExpr::Typed(TypedEntity::String(s)) => {
            println!("a string: {s}")
        }
        EvaluatedExpr::Keyword(k) => {
            println!("a keyword: {k}")
        }
        EvaluatedExpr::Typed(TypedEntity::Boolean(b)) => {
            println!("a boolean: {b}")
        }
        EvaluatedExpr::FunctionDefinition(name, pos_args, body) => {
            println!("a function definition: {name} positional args: {pos_args:?}");
            function_map.lock().usr_lib.insert(name, (pos_args, body));
        }
        EvaluatedExpr::VariableDefinition(name, var) => {
            println!("a variable definition {name:#?}");
            var_store.insert(name, var);
        }
        EvaluatedExpr::Progn(exprs) => {
            for expr in exprs {
                interpret(
                    expr,
                    function_map,
                    midi_callback_map,
                    session,
                    ruffbox,
                    sample_set,
                    var_store,
                    output_mode,
                    base_dir.clone(),
                );
            }
        }
        _ => println!("unknown"),
    }
}
