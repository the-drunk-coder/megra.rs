use crate::generator::Generator;

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};

use std::net;
use std::str::FromStr;

//crossbeam::channel::Sender<ControlMessage<BUFSIZE, NCHAN>>,
//crossbeam::channel::Receiver<ControlMessage<BUFSIZE, NCHAN>>,
/*
let (tx, rx): (
        Sender<ControlMessage<BUFSIZE, NCHAN>>,
        Receiver<ControlMessage<BUFSIZE, NCHAN>>,
    ) = crossbeam::channel::bounded(2000);
*/

pub struct VisualizerClient {
    pub host_addr: net::SocketAddrV4,
    pub to_addr: net::SocketAddrV4,
    pub socket: net::UdpSocket,
}

impl VisualizerClient {
    pub fn start() -> Self {
        let to_addr = net::SocketAddrV4::from_str("127.0.0.1:57121").unwrap();
        let host_addr = net::SocketAddrV4::from_str("127.0.0.1:57122").unwrap();
        VisualizerClient {
            host_addr,
            to_addr,
            socket: net::UdpSocket::bind(host_addr).unwrap(),
        }
    }

    pub fn create_or_update(&self, g: &Generator) {
        // switch view
        let msg_buf_add = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/graph/add".to_string(),
            args: vec![OscType::String(g.root_generator.name.clone())],
        }))
        .unwrap();
        self.socket.send_to(&msg_buf_add, self.to_addr).unwrap();

        // nodes
        for (key, label) in g.root_generator.generator.labels.iter() {
            let msg_buf_node = encoder::encode(&OscPacket::Message(OscMessage {
                addr: "/node/add".to_string(),
                args: vec![
                    OscType::String(g.root_generator.name.clone()),
                    OscType::Int(*key as i32),
                    OscType::String(label.iter().collect()),
                ],
            }))
            .unwrap();
            self.socket.send_to(&msg_buf_node, self.to_addr).unwrap();
        }
        // edges
        for (src, children) in g.root_generator.generator.children.iter() {
            for ch in children.iter() {
                let msg_buf_edge = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: "/edge/add".to_string(),
                    args: vec![
                        OscType::String(g.root_generator.name.clone()),
                        OscType::Int(*src as i32),
                        OscType::Int(ch.child_hash as i32),
                        OscType::String(ch.child.last().unwrap().to_string()),
                        OscType::Int((ch.prob * 100.0) as i32),
                    ],
                }))
                .unwrap();
                self.socket.send_to(&msg_buf_edge, self.to_addr).unwrap();
            }
        }

        // send render command ...
        let msg_buf_render = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/render".to_string(),
            args: vec![
                OscType::String(g.root_generator.name.clone()),
                OscType::String("cose".to_string()), // layout type
            ],
        }))
        .unwrap();
        self.socket.send_to(&msg_buf_render, self.to_addr).unwrap();
    }

    pub fn update_active_node(&self, g: &Generator) {
        if let Some(h) = g.root_generator.generator.current_state {
            let msg_buf_active_node = encoder::encode(&OscPacket::Message(OscMessage {
                addr: "/node/active".to_string(),
                args: vec![
                    OscType::String(g.root_generator.name.clone()),
                    OscType::Long(h as i64),
                ],
            }))
            .unwrap();
            self.socket
                .send_to(&msg_buf_active_node, self.to_addr)
                .unwrap();
        }
    }

    pub fn clear(&self, g: &Generator) {
        let msg_buf_clear = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/clear".to_string(),
            args: vec![OscType::String(g.root_generator.name.clone())],
        }))
        .unwrap();
        self.socket.send_to(&msg_buf_clear, self.to_addr).unwrap();
    }
}
