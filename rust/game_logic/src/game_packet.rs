use std::error::Error;
use std::f32;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::string::ParseError;
use std::sync::mpsc::{Sender, SendError};
use std::time::Instant;
use log::{debug, error};
use crate::players_state::{PlayerInput, PlayerMessage, PlayersStateMessage};
use crate::game_state::GameState;

const CONSTANT_VALUE: u16 = 45_581;
const GAME_TYPE_STATE: u16 = 2;
const GAME_TYPE_PONG: u16 = 1;

const PACKET_IN_LEN: usize = 32;
const PACKET_OUT_LEN: usize = 64;


pub enum PacketType {
    ClientHello(u32),
    ClientPing(u32),
    ClientInput(u32, ),
    ServerGameState,
}

struct PacketIn {
    const_val: u16,
    player_id: u64,
    message_id: u32,
    touch_vec_x: f32,
    touch_vec_y: f32,
    unused: [u8; 10]
}

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

pub fn handle_socket(state_sender: Sender<PlayersStateMessage>, socket: UdpSocket) {
    let mut buf = [0; 32];

    loop {
        // println!("Reading socket...");
        match socket.recv_from(&mut buf) {
            Ok((len, client_address)) => {
                let i = Instant::now();
                let packet = decode_inbound_packet(len, &buf);
                match packet {
                    Ok(p) => {
                        match state_sender.send(PlayersStateMessage::PlayerInput(PlayerMessage { player_id: p.player_id, player_socket: client_address, input: PlayerInput { vec_x: p.touch_vec_x, vec_y: p.touch_vec_y } })) {
                            Ok(val) => {}
                            Err(err) => {
                                error!("Cannot send: {}", err);
                            }
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

    // debug!("{} bytes read, message: {:?}", len, bytes);

    let packet = PacketIn {
        const_val: u16::from_ne_bytes(bytes[0..2].try_into()?),
        player_id: u64::from_ne_bytes(bytes[2..10].try_into()?),
        message_id: u32::from_ne_bytes(bytes[10..14].try_into()?),
        touch_vec_x: f32::from_ne_bytes(bytes[14..18].try_into()?),
        touch_vec_y: f32::from_ne_bytes(bytes[18..22].try_into()?),
        unused: bytes[22..32].try_into()?,
    };

    if packet.const_val != CONSTANT_VALUE {
        return Err(Box::from("Wrong constant value"));
    }

    Ok(packet)
}
