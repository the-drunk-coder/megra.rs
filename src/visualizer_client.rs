use crate::generator::Generator;
use crate::session::Session;

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

    pub fn create_or_update(g: &Generator) {
	
    }

    pub fn update_active_node(g: &Generator) {
	
    }
    
    pub fn clear(g: &Generator) {
	
    }
}




