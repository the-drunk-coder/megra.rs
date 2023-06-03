use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync;

use ruffbox_synth::ruffbox::RuffboxControls;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::builtin_types::*;
use crate::interpreter;
//use crate::parser;
use crate::parser;
use crate::parser::FunctionMap;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::{OutputMode, Session};

#[allow(clippy::too_many_arguments)]
pub fn start_repl<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    midi_callback_map: &sync::Arc<Mutex<HashMap<u8, Command>>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    var_store: &sync::Arc<VariableStore>,
    mode: OutputMode,
    base_dir: String,
) -> Result<(), anyhow::Error> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("megra>> ");
        match readline {
            Ok(line) => {
                // ignore empty lines ...
                if line.is_empty() {
                    continue;
                }

                let pfa_in = parser::eval_from_str(
                    line.as_str(),
                    &mut function_map.lock(),
                    var_store,
                    sample_set,
                    mode,
                );

                match pfa_in {
                    Err(e) => {
                        // this needs a more elegant way ... like,
                        // retrieve the actual error from the parser, instead
                        // of looking for the string ...
                        // if the error is that a closing paren is missing,
                        // assume we're waiting for more lines.
                        // once a complete input is found,
                        if e.contains("closing paren") {
                            let mut line_buffer: String = "".to_string();
                            line_buffer.push_str(line.as_str());
                            loop {
                                let readline_inner = rl.readline(".. ");
                                match readline_inner {
                                    Ok(line) => {
                                        line_buffer.push_str(line.as_str());
                                        let inner_pfa_in = parser::eval_from_str(
                                            line_buffer.as_str(),
                                            &mut function_map.lock(),
                                            var_store,
                                            sample_set,
                                            mode,
                                        );
                                        match inner_pfa_in {
                                            Ok(pfa) => {
                                                interpreter::interpret(
                                                    pfa,
                                                    function_map,
                                                    midi_callback_map,
                                                    session,
                                                    ruffbox,
                                                    sample_set,
                                                    var_store,
                                                    mode,
                                                    base_dir.clone(),
                                                );
                                                rl.add_history_entry(line_buffer.as_str());
                                                break;
                                            }
                                            Err(_) => {
                                                // wait for more input ...
                                                continue;
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }
                        } else {
                            println!("parser error {e}");
                        }
                    }
                    Ok(pfa) => {
                        interpreter::interpret(
                            pfa,
                            function_map,
                            midi_callback_map,
                            session,
                            ruffbox,
                            sample_set,
                            var_store,
                            mode,
                            base_dir.clone(),
                        );
                        rl.add_history_entry(line.as_str());
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();
    Ok(())
}
