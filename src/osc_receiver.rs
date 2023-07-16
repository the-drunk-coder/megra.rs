use parking_lot::Mutex;
use rosc::OscPacket;
use ruffbox_synth::ruffbox::RuffboxControls;

use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync;

use crate::builtin_types::VariableStore;
use crate::callbacks::{CallbackKey, CallbackMap};
use crate::interpreter;
use crate::parser::FunctionMap;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::{OutputMode, Session};

pub struct OscReceiver;

impl OscReceiver {
    pub fn start_receiver_thread_udp<const BUFSIZE: usize, const NCHAN: usize>(
        target: String,
        function_map: sync::Arc<Mutex<FunctionMap>>,
        callback_map: sync::Arc<CallbackMap>, // could be dashmap i suppose
        session: sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        sample_set: sync::Arc<Mutex<SampleAndWavematrixSet>>,
        var_store: sync::Arc<VariableStore>,
        mode: OutputMode,
        base_dir: String,
    ) {
        let addr = match SocketAddrV4::from_str(&target) {
            Ok(addr) => addr,
            Err(_) => panic!("err"),
        };

        let sock = UdpSocket::bind(addr).unwrap();

        println!("Listening to {}", addr);

        let mut buf = [0u8; rosc::decoder::MTU];

        std::thread::spawn(move || loop {
            match sock.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    println!("Received packet with size {} from: {}", size, addr);
                    let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                    match packet {
                        OscPacket::Message(msg) => {
                            println!("OSC address: {}", msg.addr);
                            println!("OSC arguments: {:?}", msg.args);
                            if let Some(thing) = callback_map.get(&CallbackKey::OscAddr(msg.addr)) {
                                interpreter::interpret(
                                    thing.clone(),
                                    &function_map,
                                    &callback_map,
                                    &session,
                                    &ruffbox,
                                    &sample_set,
                                    &var_store,
                                    mode,
                                    base_dir.clone(),
                                );
                            } else {
                                println!("no callback for OSC addr ??");
                            }
                        }
                        OscPacket::Bundle(bundle) => {
                            println!("OSC Bundle: {:?}", bundle);
                        }
                    }
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                    break;
                }
            }
        });
    }
}
