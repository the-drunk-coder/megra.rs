use parking_lot::Mutex;
use std::sync;
use std::thread;

use ruffbox_synth::ruffbox::Ruffbox;

use crate::builtin_types::*;
use crate::commands;
use crate::sample_set::SampleSet;
use crate::session::{OutputMode, Session};

pub fn interpret<const BUFSIZE: usize, const NCHAN: usize>(
    parsed_in: Expr,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    output_mode: OutputMode,
) {
    match parsed_in {
        Expr::Comment => {
            println!("a comment")
        }
        Expr::Constant(Atom::Generator(g)) => {
            print!("a generator called \'");
            for tag in g.id_tags.iter() {
                print!("{} ", tag);
            }
            println!("\'");
        }
        Expr::Constant(Atom::Parameter(_)) => {
            println!("a parameter");
        }
        Expr::Constant(Atom::SoundEvent(_)) => {
            println!("a sound event");
        }
        Expr::Constant(Atom::ControlEvent(_)) => {
            println!("a control event");
        }
        Expr::Constant(Atom::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorModifierFunction(_),
        )) => {
            println!("a gen mod fun");
        }
        Expr::Constant(Atom::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorProcessor(_),
        )) => {
            println!("a gen proc");
        }
        Expr::Constant(Atom::GeneratorList(gl)) => {
            println!("a gen list");
            for gen in gl.iter() {
                print!("--- a generator called \'");
                for tag in gen.id_tags.iter() {
                    print!("{} ", tag);
                }
                println!("\'");
            }
        }
        Expr::Constant(Atom::SyncContext(mut s)) => {
            println!("a context called \'{}\'", s.name);
            Session::handle_context(
                &mut s,
                &session,
                &ruffbox,
                parts_store,
                &global_parameters,
                output_mode,
            );
        }
        Expr::Constant(Atom::Command(c)) => {
            match c {
                Command::Clear => {
                    let session2 = sync::Arc::clone(session);
                    let parts_store2 = sync::Arc::clone(parts_store);
                    thread::spawn(move || {
                        Session::clear_session(&session2, &parts_store2);
                        println!("a command (stop session)");
                    });
                }
                Command::LoadSample((set, mut keywords, path)) => {
                    let ruffbox2 = sync::Arc::clone(ruffbox);
                    let sample_set2 = sync::Arc::clone(sample_set);
                    thread::spawn(move || {
                        commands::load_sample(&ruffbox2, &sample_set2, set, &mut keywords, path);
                        println!("a command (load sample)");
                    });
                }
                Command::LoadSampleSets(path) => {
                    let ruffbox2 = sync::Arc::clone(ruffbox);
                    let sample_set2 = sync::Arc::clone(sample_set);
                    thread::spawn(move || {
                        commands::load_sample_sets(&ruffbox2, &sample_set2, path);
                        println!("a command (load sample sets)");
                    });
                }
                Command::LoadSampleSet(path) => {
                    let ruffbox2 = sync::Arc::clone(ruffbox);
                    let sample_set2 = sync::Arc::clone(sample_set);
                    thread::spawn(move || {
                        commands::load_sample_set_string(&ruffbox2, &sample_set2, path);
                        println!("a command (load sample sets)");
                    });
                }
                Command::LoadPart((name, part)) => {
                    commands::load_part(parts_store, name, part);
                    println!("a command (load part)");
                }
                Command::FreezeBuffer(freezbuf) => {
                    commands::freeze_buffer(ruffbox, freezbuf);
                    println!("freeze buffer");
                }
                Command::Tmod(p) => {
                    commands::set_global_tmod(global_parameters, p);
                }
                Command::GlobRes(v) => {
                    commands::set_global_lifemodel_resources(global_parameters, v);
                }
                Command::GlobalRuffboxParams(m) => {
                    commands::set_global_ruffbox_parameters(ruffbox, &m);
                }
                Command::ExportDot((f, g)) => {
                    commands::export_dot(&f, &g);
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
            };
        }
        Expr::Constant(Atom::Float(f)) => {
            println!("a number: {}", f)
        }
        _ => println!("unknown"),
    }
}
