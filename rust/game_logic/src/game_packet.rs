use std::error::Error;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::string::ParseError;
use std::sync::mpsc::{Sender, SendError};
use std::time::Instant;
use crate::players_state::{PlayerInput, PlayerMessage, PlayersStateMessage};
use crate::{PlayerId};

// todo maybe better add some unused zeros at the end? and send 32 bytes or even 64 bytes?
const PACKET_IN_LEN: usize = 32;
const CONSTANT_VALUE: u16 = 45_581;

pub enum PacketType {
    ClientHello(u32),
    ClientPing(u32),
    ClientInput(u32, ),
    ServerGameState,
}

struct PacketIn {
    const_val: u16,
    packet_type: u16,
    player_id: PlayerId,
    message_id: u32,
    start_x: i16,
    start_y: i16,
    current_x: i16,
    current_y: i16,
    unused: [u8; 12]
}

pub fn handle_socket(input_sender: Sender<PlayersStateMessage>, socket: UdpSocket) {
    let mut buf = [0; 32];

    let mut player_waiting: Option<PlayerId> = None;

    loop {
        println!("Reading socket...");
        match socket.recv_from(&mut buf) {
            Ok((len, client_address)) => {
                let i = Instant::now();
                let packet = decode_inbound_packet(len, &buf);
                match packet {
                    Ok(p) => {
                        let player_id = client_address;
                        match input_sender.send(PlayersStateMessage::PlayerInput(PlayerMessage {player_id,  input: PlayerInput{ start_x: p.start_x as f32, start_y: p.start_y as f32, current_x: p.current_x as f32, current_y: p.current_y as f32 }})) {
                            Ok(val) => {}
                            Err(err) => {
                                println!("Cannot send: {}", err);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error decoding packet: {}", e);
                    }
                }
                println!("handling_socket takes {:?}", i.elapsed());
            }
            Err(e) => {
                println!("Error kind: {}, error: {}", e.kind(), e);
                println!("Local addr, peer addr: {:?} {:?}", socket.local_addr(), socket.peer_addr());
                panic!("error");
            }
        }
    }
}

fn decode_inbound_packet(len: usize, bytes: &[u8; 32]) -> Result<PacketIn, Box<dyn Error>> {
    if len != 32 {
        return Err(Box::from("Wrong packet len"));
    }

    println!("{} bytes read, message: {:?}", len, bytes);

    let packet = PacketIn {
        const_val: u16::from_ne_bytes(bytes[0..2].try_into()?),
        packet_type: u16::from_ne_bytes(bytes[2..4].try_into()?),
        player_id: u32::from_ne_bytes(bytes[4..8].try_into()?),
        message_id: u32::from_ne_bytes(bytes[8..12].try_into()?),
        start_x: i16::from_ne_bytes(bytes[12..14].try_into()?),
        start_y: i16::from_ne_bytes(bytes[14..16].try_into()?),
        current_x: i16::from_ne_bytes(bytes[16..18].try_into()?),
        current_y: i16::from_ne_bytes(bytes[18..20].try_into()?),
        unused: bytes[20..32].try_into()?,
    };

    if packet.const_val != CONSTANT_VALUE {
        return Err(Box::from("Wrong constant value"));
    }

    Ok(packet)
}
