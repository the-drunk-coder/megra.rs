use parking_lot::Mutex;
use rosc::{OscPacket, OscType};
use ruffbox_synth::ruffbox::RuffboxControls;

use std::collections::HashMap;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync;

use crate::builtin_types::{Comparable, GlobalVariables, TypedEntity};
use crate::interpreter;
use crate::parser::{eval_expression, EvaluatedExpr, FunctionMap};
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::{OutputMode, Session};

pub struct OscReceiver;

impl OscReceiver {
    #[allow(clippy::too_many_arguments)]
    pub fn start_receiver_thread_udp<const BUFSIZE: usize, const NCHAN: usize>(
        target: String,
        function_map: sync::Arc<Mutex<FunctionMap>>,
        session: Session<BUFSIZE, NCHAN>,
        ruffbox: sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        sample_set: SampleAndWavematrixSet,
        globals: sync::Arc<GlobalVariables>,
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

                            // check whether we have an OSC function stored under this address ...
                            let functions = function_map.lock();

                            if functions.usr_lib.contains_key(&msg.addr) {
                                let (fun_arg_names, fun_expr) =
                                    functions.usr_lib.get(&msg.addr).unwrap().clone();

                                // FIRST, eval local args,
                                // manual zip
                                let mut local_args = HashMap::new();
                                for (i, val) in msg.args[..fun_arg_names.len()].iter().enumerate() {
                                    // TODO: better type handling ...
                                    local_args.insert(
                                        fun_arg_names[i].clone(),
                                        match val {
                                            OscType::Float(f) => EvaluatedExpr::Typed(
                                                TypedEntity::Comparable(Comparable::Float(*f)),
                                            ),
                                            OscType::Double(d) => {
                                                EvaluatedExpr::Typed(TypedEntity::Comparable(
                                                    Comparable::Float(*d as f32),
                                                ))
                                            }
                                            OscType::Int(i) => {
                                                EvaluatedExpr::Typed(TypedEntity::Comparable(
                                                    Comparable::Float(*i as f32),
                                                ))
                                            }
                                            OscType::Long(i) => {
                                                EvaluatedExpr::Typed(TypedEntity::Comparable(
                                                    Comparable::Float(*i as f32),
                                                ))
                                            }
                                            OscType::String(s) => {
                                                EvaluatedExpr::Typed(TypedEntity::Comparable(
                                                    Comparable::String(s.clone()),
                                                ))
                                            }
                                            _ => {
                                                continue;
                                            } // dirty ..
                                        },
                                    );
                                }

                                // THIRD
                                if let Some(fun_tail) = fun_expr
                                    .iter()
                                    .map(|expr| {
                                        eval_expression(
                                            expr,
                                            &functions,
                                            &globals,
                                            Some(&local_args),
                                            sample_set.clone(),
                                            mode,
                                        )
                                    })
                                    .collect::<Option<Vec<EvaluatedExpr>>>()
                                {
                                    // return last form result, cl-style
                                    for eval_expr in fun_tail {
                                        interpreter::interpret(
                                            eval_expr,
                                            &function_map,
                                            session.clone(),
                                            &ruffbox,
                                            sample_set.clone(),
                                            &globals,
                                            mode,
                                            base_dir.clone(),
                                        );
                                    }
                                }
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
                    //break;
                }
            }
        });
    }
}
