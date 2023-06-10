use rosc::OscPacket;

use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

pub struct OscReceiver {}

impl OscReceiver {
    pub fn start_receiver_thread_udp(&mut self, target: String) {
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
                    handle_packet(packet);
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                    break;
                }
            }
        });
    }
}

fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            println!("OSC arguments: {:?}", msg.args);
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}
