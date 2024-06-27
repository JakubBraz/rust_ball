use std::error::Error;
use std::f32;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::string::ParseError;
use std::sync::mpsc::{Sender, SendError};
use std::time::Instant;
use log::error;
use crate::players_state::{PlayerInput, PlayerMessage, PlayersStateMessage};
use crate::game_state::GameState;
use crate::ping_handler;
use crate::ping_handler::PingMessage;

const CONSTANT_VALUE: u16 = 45_581;
const GAME_TYPE_STATE: u16 = 2;
const GAME_TYPE_PONG: u16 = 1;

const PACKET_IN_LEN: usize = 32;
const PACKET_OUT_LEN: usize = 64;

// todo move it out of here
type PlayerId = u32;

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
    touch_vec_x: f32,
    touch_vec_y: f32,
    unused: [u8; 12]
}

// #[derive(Default)]
// pub struct PacketOut {
//     const_val: u16,
//     packet_type: u16,
//     seq_num: u32,
//     // ball_x: f32,
//     // ball_y: f32,
//     // player_1_1_x: f32,
//     // player_1_1_y: f32,
//     // player_1_2_x: f32,
//     // player_1_2_y: f32,
//     // player_2_1_x: f32,
//     // player_2_1_y: f32,
//     // player_2_2_x: f32,
//     // player_2_2_y: f32,
//     // score1: u16,
//     // score2: u16,
//     // time: u16,
//     // kicks_game_over_bool: u8,
//     unused: [u8; 9],
// }
//
// impl PacketOut {
//     pub fn pong() -> PacketOut {
//         let mut p: PacketOut = Default::default();
//         p.packet_type = 1;
//         p
//     }
//
//     pub fn state(state: GameState) -> PacketOut {
//         let mut p = PacketOut::default();
//         p.packet_type = 2;
//         p
//     }
// }

pub fn to_bytes(gs: &GameState, seq_num: u32) -> [u8; PACKET_OUT_LEN] {
    let score1: u16 = 0;
    let score2: u16 = 0;
    let time: u16 = 0;
    let unused: [u8; 9] = [0; 9];

    let is_left = (if gs.is_left { 1 } else { 0 }) << 7;
    let kicks_game_over_bool: u8 = is_left;

    let mut out_packet = [
        0, 0, // const val
        0, 0, // game type
        0, 0, 0, 0, // seq num
        0, 0, 0, 0, // ball x
        0, 0, 0, 0, // ball y
        0, 0, 0, 0, // player 1 1 x
        0, 0, 0, 0, // player 1 1 y
        0, 0, 0, 0, // player 1 2 x
        0, 0, 0, 0, // player 1 2 y
        0, 0, 0, 0, // player 2 1 x
        0, 0, 0, 0, // player 2 1 y
        0, 0, 0, 0, // player 2 2 x
        0, 0, 0, 0, // player 2 2 y
        0, 0, // score 1
        0, 0, // score 2
        0, 0, // time
        0, // boolean flags, is_left | kick ready 1 2 3 4 | is game over
        0, 0, 0, 0, 0, 0, 0, 0, 0 // 9 unused bytes
    ];

    out_packet[0..2].clone_from_slice(CONSTANT_VALUE.to_le_bytes().as_slice());
    out_packet[2..4].clone_from_slice(GAME_TYPE_STATE.to_le_bytes().as_slice());
    out_packet[4..8].clone_from_slice(seq_num.to_le_bytes().as_slice());
    out_packet[8..12].clone_from_slice(gs.ball_x.to_le_bytes().as_slice());
    out_packet[12..16].clone_from_slice(gs.ball_y.to_le_bytes().as_slice());
    out_packet[16..20].clone_from_slice(gs.player_1_1_x.to_le_bytes().as_slice());
    out_packet[20..24].clone_from_slice(gs.player_1_1_y.to_le_bytes().as_slice());
    out_packet[24..28].clone_from_slice(gs.player_1_2_x.to_le_bytes().as_slice());
    out_packet[28..32].clone_from_slice(gs.player_1_2_y.to_le_bytes().as_slice());
    out_packet[32..36].clone_from_slice(gs.player_2_1_x.to_le_bytes().as_slice());
    out_packet[36..40].clone_from_slice(gs.player_2_1_y.to_le_bytes().as_slice());
    out_packet[40..44].clone_from_slice(gs.player_2_2_x.to_le_bytes().as_slice());
    out_packet[44..48].clone_from_slice(gs.player_2_2_y.to_le_bytes().as_slice());
    out_packet[48..50].clone_from_slice(score1.to_le_bytes().as_slice());
    out_packet[50..52].clone_from_slice(score2.to_le_bytes().as_slice());
    out_packet[52..54].clone_from_slice(time.to_le_bytes().as_slice());
    out_packet[54..55].clone_from_slice(kicks_game_over_bool.to_le_bytes().as_slice());
    out_packet[55..].clone_from_slice(unused.as_slice());

    let v = [
        CONSTANT_VALUE.to_le_bytes().as_slice(),
        GAME_TYPE_STATE.to_le_bytes().as_slice(),
        seq_num.to_le_bytes().as_slice(),
        gs.ball_x.to_le_bytes().as_slice(),
        gs.ball_y.to_le_bytes().as_slice(),
        gs.player_1_1_x.to_le_bytes().as_slice(),
        gs.player_1_1_y.to_le_bytes().as_slice(),
        gs.player_1_2_x.to_le_bytes().as_slice(),
        gs.player_1_2_y.to_le_bytes().as_slice(),
        gs.player_2_1_x.to_le_bytes().as_slice(),
        gs.player_2_1_y.to_le_bytes().as_slice(),
        gs.player_2_2_x.to_le_bytes().as_slice(),
        gs.player_2_2_y.to_le_bytes().as_slice(),
        score1.to_le_bytes().as_slice(),
        score2.to_le_bytes().as_slice(),
        time.to_le_bytes().as_slice(),
        kicks_game_over_bool.to_le_bytes().as_slice(),
        unused.as_slice()
    ].concat();

    out_packet
}

pub fn handle_socket(input_sender: Sender<PlayersStateMessage>, pong_sender: Sender<ping_handler::PingMessage>, socket: UdpSocket) {
    let mut buf = [0; 32];

    loop {
        // println!("Reading socket...");
        match socket.recv_from(&mut buf) {
            Ok((len, client_address)) => {
                let i = Instant::now();
                let packet = decode_inbound_packet(len, &buf);
                match packet {
                    Ok(p) => {
                        if p.packet_type == GAME_TYPE_PONG {
                            match pong_sender.send(PingMessage::PingReceived(client_address)) {
                                Ok(_) => {}
                                Err(e) => error!("Cannot send ping message, error: {}", e)
                            };
                        }
                        else if p.packet_type == GAME_TYPE_STATE {
                            let player_id = client_address;
                            match input_sender.send(PlayersStateMessage::PlayerInput(PlayerMessage { player_socket: player_id, input: PlayerInput { vec_x: p.touch_vec_x, vec_y: p.touch_vec_y } })) {
                                Ok(val) => {}
                                Err(err) => {
                                    error!("Cannot send: {}", err);
                                }
                            }
                        }
                        else {
                            error!("Unknown packet type: {}", p.packet_type);
                        }
                    }
                    Err(e) => {
                        error!("Error decoding packet: {}", e);
                    }
                }
                // println!("handling_socket takes {:?}", i.elapsed());
            }
            Err(e) => {
                error!("Error kind: {}, error: {}", e.kind(), e);
                error!("Local addr, peer addr: {:?} {:?}", socket.local_addr(), socket.peer_addr());
                panic!("error");
            }
        }
    }
}

fn decode_inbound_packet(len: usize, bytes: &[u8; 32]) -> Result<PacketIn, Box<dyn Error>> {
    if len != 32 {
        return Err(Box::from("Wrong packet len"));
    }

    // println!("{} bytes read, message: {:?}", len, bytes);

    let packet = PacketIn {
        const_val: u16::from_ne_bytes(bytes[0..2].try_into()?),
        packet_type: u16::from_ne_bytes(bytes[2..4].try_into()?),
        player_id: u32::from_ne_bytes(bytes[4..8].try_into()?),
        message_id: u32::from_ne_bytes(bytes[8..12].try_into()?),
        touch_vec_x: f32::from_ne_bytes(bytes[12..16].try_into()?),
        touch_vec_y: f32::from_ne_bytes(bytes[16..20].try_into()?),
        unused: bytes[20..32].try_into()?,
    };

    if packet.const_val != CONSTANT_VALUE {
        return Err(Box::from("Wrong constant value"));
    }

    Ok(packet)
}

pub fn pong_message() -> [u8; PACKET_OUT_LEN]{
    let mut bytes: [u8; PACKET_OUT_LEN] = [0; PACKET_OUT_LEN];

    // out_packet[0..2].clone_from_slice(CONSTANT_VALUE.to_le_bytes().as_slice());
    // out_packet[2..4].clone_from_slice(GAME_TYPE_STATE.to_le_bytes().as_slice());

    bytes[0..2].clone_from_slice(&CONSTANT_VALUE.to_le_bytes());
    bytes[2..4].clone_from_slice(&GAME_TYPE_PONG.to_le_bytes());
    bytes
}
