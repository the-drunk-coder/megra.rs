use midir::{Ignore, MidiInput};

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync;

use crate::{Command, GlobalParameters, OutputMode, PartsStore, SampleAndWavematrixSet, Session};

use crate::commands;

use ruffbox_synth::ruffbox::RuffboxControls;

pub fn list_midi_input_ports() -> Result<(), anyhow::Error> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);
    println!("\nAvailable input ports:");
    let in_ports = midi_in.ports();
    for (i, p) in in_ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p).unwrap());
    }
    Ok(())
}

pub fn open_midi_input_port<const BUFSIZE: usize, const NCHAN: usize>(
    midi_callback_map: sync::Arc<Mutex<HashMap<u8, Command>>>, // could be dashmap i suppose
    in_port_num: usize,
    session: sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    global_parameters: sync::Arc<GlobalParameters>,
    sample_set: sync::Arc<Mutex<SampleAndWavematrixSet>>,
    parts_store: sync::Arc<Mutex<PartsStore>>,
    output_mode: OutputMode,
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
                const NOTE_ON_MSG: u8 = 145;
                if message[0] == NOTE_ON_MSG {
                    let key = message[1];
                    if let Some(command) = midi_callback_map.lock().get(&key) {
                        interpret_midi_command(
                            command.clone(),
                            &session,
                            &ruffbox,
                            &global_parameters,
                            &sample_set,
                            &parts_store,
                            output_mode,
                        );
                    } else {
                        println!("EMPTY KEY {:?} (len = {})", message, message.len());
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

fn interpret_midi_command<const BUFSIZE: usize, const NCHAN: usize>(
    c: Command,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    output_mode: OutputMode,
) {
    match c {
        // I should overthink this in general ...
        Command::Once((mut s, mut c)) => {
            commands::once(
                ruffbox,
                parts_store,
                global_parameters,
                sample_set,
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
                sample_set,
                session,
                output_mode,
                name,
            );
        }
        _ => {
            println!("this command isn't midi-enabled !")
        }
    }
}
