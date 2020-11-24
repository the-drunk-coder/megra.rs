use crate::session::Session;
use crate::parser::SampleSet;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::parser;
use crate::interpreter;
use ruffbox_synth::ruffbox::Ruffbox;
use std::sync;
use parking_lot::Mutex;

pub fn start_repl(ruffbox: &sync::Arc<Mutex<Ruffbox<512>>>) -> Result<(), anyhow::Error> {
    let mut session = Session::new();
    let mut sample_set = SampleSet::new();
    
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
		if line.len() == 0 { continue; }
		
		let pfa_in = parser::eval_from_str(&line.as_str(), &sample_set);
		
		match pfa_in {
		    Err(e) => {

			// this needs a more elegant way ... like,
			// retrieve the actual error from the parser, instead
			// of looking for the string ...
			// if the error is that a closing paren is missing,
			// assume we're waiting for more lines.
			// once a complete input is found, 
			if e.contains("closing paren") {
			    let mut line_buffer:String = "".to_string();
			    line_buffer.push_str(&line.as_str());
			    loop {
				let readline_inner = rl.readline(".. ");
				match readline_inner {
				    Ok(line) => {
					line_buffer.push_str(&line.as_str());
					let inner_pfa_in = parser::eval_from_str(&line_buffer.as_str(), &sample_set);
					match inner_pfa_in {
					    Ok(pfa) => {
						interpreter::interpret(&mut session, &mut sample_set, pfa, &ruffbox);			
						rl.add_history_entry(line_buffer.as_str());
						break;
					    },
					    Err(_) => {
						// wait for more input ...
						continue;
					    }
					}
				    },
				    Err(_) => {
					break;
				    }
				}				
			    }
			    
			} else {
			    println!("parser error {}", e);
			}
		    },
		    Ok(pfa) => {
			interpreter::interpret(&mut session, &mut sample_set, pfa, &ruffbox);			
			rl.add_history_entry(line.as_str());						
		    }
		}
	    },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    rl.save_history("history.txt").unwrap();
    Ok(())
}
