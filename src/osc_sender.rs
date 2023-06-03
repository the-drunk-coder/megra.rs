use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};

use std::net;
use std::str::FromStr;

pub struct OscSender {
    pub host_addr: net::SocketAddrV4,
    pub to_addr: net::SocketAddrV4,
    pub socket: net::UdpSocket,
}

impl OscSender {
    pub fn start(target: String, host: String) -> Result<Self, anyhow::Error> {
        let to_addr = net::SocketAddrV4::from_str(&target)?;
        let host_addr = net::SocketAddrV4::from_str(&host)?;
        let socket = net::UdpSocket::bind(host_addr)?;
        Ok(OscSender {
            host_addr,
            to_addr,
            socket,
        })
    }

    pub fn send_message(&self, addr: String, args: Vec<OscType>) -> Result<(), anyhow::Error> {
        let msg_buf_add = encoder::encode(&OscPacket::Message(OscMessage { addr, args }))?;
        self.socket.send_to(&msg_buf_add, self.to_addr)?;
        Ok(())
    }
}
