use crate::generator::Generator;

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};

use std::net;
use std::str::FromStr;

pub struct VisualizerClient {
    pub host_addr : net::SocketAddrV4,
    pub to_addr : net::SocketAddrV4,
    pub socket : net::UdpSocket,
}

impl VisualizerClient {
    pub fn start() -> Self {
	let to_addr = net::SocketAddrV4::from_str("127.0.0.1:57121").unwrap();
	let host_addr = net::SocketAddrV4::from_str("127.0.0.1:57122").unwrap();
	VisualizerClient {
	    host_addr,
	    to_addr,
	    socket : net::UdpSocket::bind(host_addr).unwrap()
	}	
    }

    pub fn create_or_update(&self, g: &Generator) {
	// switch view
	let msg_buf_add = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/graph/add".to_string(),
            args: vec![OscType::String(g.root_generator.name.clone())],
	})).unwrap();

	self.socket.send_to(&msg_buf_add, self.to_addr).unwrap();
    }

    pub fn update_active_node(g: &Generator) {
	
    }
    
    pub fn clear(g: &Generator) {
	
    }
}




