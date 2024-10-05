use anyhow::Result;
use midir::{Ignore, MidiInput};

use std::collections::HashMap;

use crate::builtin_types::Comparable;
use crate::parser::{eval_expression, EvaluatedExpr};
use crate::{interpreter, Session};

pub fn list_midi_input_ports() {
    if let Ok(mut midi_in) = MidiInput::new("midir reading input") {
        midi_in.ignore(Ignore::None);
        println!("\nAvailable input ports:");
        let in_ports = midi_in.ports();
        for (i, p) in in_ports.iter().enumerate() {
            println!("{}: {}", i, midi_in.port_name(p).unwrap());
        }
    }
}

pub fn open_midi_input_port<const BUFSIZE: usize, const NCHAN: usize>(
    in_port_num: usize,
    session: Session<BUFSIZE, NCHAN>,
    base_dir: String,
) {
    let mut midi_in = MidiInput::new("midir reading input").unwrap();
    midi_in.ignore(Ignore::None);
    let in_ports = midi_in.ports();
    let in_port = in_ports
        .get(in_port_num)
        .ok_or("invalid input port selected")
        .unwrap();

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port).unwrap();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in
        .connect(
            in_port,
            "midir-read-input",
            move |_, message, _| {
                if session.functions.usr_lib.contains_key("midi") {
                    let (fun_arg_names, fun_expr) =
                        session.functions.usr_lib.get("midi").unwrap().clone();

                    // FIRST, eval local args,
                    // manual zip
                    let mut local_args = HashMap::new();
                    for (i, val) in fun_arg_names.iter().enumerate() {
                        if i < message.len() {
                            local_args.insert(
                                val.clone(),
                                EvaluatedExpr::Typed(
                                    crate::builtin_types::TypedEntity::Comparable(
                                        Comparable::Float(message[i] as f32),
                                    ),
                                ),
                            );
                        }
                        //else {
                        //  println!("no midi arg available for pos arg {val}");
                        // }
                    }

                    // THIRD
                    if let Ok(fun_tail) = fun_expr
                        .iter()
                        .map(|expr| {
                            eval_expression(
                                expr,
                                &session.functions,
                                &session.globals,
                                None,
                                session.sample_set.clone(),
                                session.output_mode,
                            )
                        })
                        .collect::<Result<Vec<EvaluatedExpr>>>()
                    {
                        // return last form result, cl-style
                        for eval_expr in fun_tail {
                            interpreter::interpret(eval_expr, session.clone(), base_dir.clone());
                        }
                    }
                }
            },
            (),
        )
        .unwrap();

    println!("Connection open, reading input from '{in_port_name}' ...");

    // keep midi thread running until we quit the program ...
    std::thread::park();
}
