use midir::{Ignore, MidiInput};

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

pub fn open_midi_input_port(in_port_num: usize) {
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
            move |stamp, message, _| {
                println!("{}: {:?} (len = {})", stamp, message, message.len());
            },
            (),
        )
        .unwrap();

    println!("Connection open, reading input from '{}' ...", in_port_name);

    // keep midi thread running until we quit the program ...
    std::thread::park();
}
