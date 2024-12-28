use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::interpreter;

use crate::session::Session;

pub fn start_repl<const BUFSIZE: usize, const NCHAN: usize>(
    session: Session<BUFSIZE, NCHAN>,
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

                let pfa_in = crate::eval::parse_and_eval_from_str(
                    line.as_str(),
                    &session.functions,
                    &session.globals,
                    session.sample_set.clone(),
                    session.output_mode,
                );

                match pfa_in {
                    Err(e) => {
                        // this needs a more elegant way ... like,
                        // retrieve the actual error from the parser, instead
                        // of looking for the string ...
                        // if the error is that a closing paren is missing,
                        // assume we're waiting for more lines.
                        // once a complete input is found,
                        if e.to_string().contains("closing paren") {
                            let mut line_buffer: String = "".to_string();
                            line_buffer.push_str(line.as_str());
                            loop {
                                let readline_inner = rl.readline(".. ");
                                match readline_inner {
                                    Ok(line) => {
                                        line_buffer.push_str(line.as_str());
                                        let inner_pfa_in = crate::eval::parse_and_eval_from_str(
                                            line_buffer.as_str(),
                                            &session.functions,
                                            &session.globals,
                                            session.sample_set.clone(),
                                            session.output_mode,
                                        );
                                        match inner_pfa_in {
                                            Ok(pfa) => {
                                                interpreter::interpret(
                                                    pfa,
                                                    session.clone(),
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
                        interpreter::interpret(pfa, session.clone(), base_dir.clone());
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
