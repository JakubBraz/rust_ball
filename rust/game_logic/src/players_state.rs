use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, RecvError, Sender, SendError};
use std::time::{Duration, Instant};
use log::{debug, error, info};
use rand::random;
use crate::game_packet;

const TIME_TO_REMOVE_PING: Duration = Duration::from_secs(30);

pub enum PlayerGame {
    NewPlayer,
    GetSocket(u32),
    SetSocket(u32, SocketAddr),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PlayerInput {
    pub vec_x: f32,
    pub vec_y: f32,
}

#[derive(Debug)]
pub struct PlayerMessage {
    // todo socket addr as player id, is it a good idea?
    pub player_socket: SocketAddr,
    pub input: PlayerInput,
}

#[derive(Debug)]
pub enum PlayersStateMessage {
    PlayerInput(PlayerMessage),
    GetGameId(Sender<Option<(u32, Vec<(SocketAddr, PlayerInput, bool)>)>>),
    PlayerStateMonitor,
    CheckPings,
    PingReceived(SocketAddr, u16),
}

pub fn handle_players_state(rx: Receiver<PlayersStateMessage>, socket: UdpSocket) {
    // todo it is overcomplicated
    let mut inputs: HashMap<SocketAddr, PlayerInput> = HashMap::new();

    let mut boards: HashMap<u32, Vec<(SocketAddr, bool)>> = HashMap::new();
    let mut waiting_boards: HashMap<u16, (SocketAddr, u32)> = HashMap::new();
    // let mut waiting_board_id: Option<u32> = None;
    let mut boards_to_update: VecDeque<u32> = VecDeque::new();

    // todo maybe merge it into one hashmap with "inputs"?
    let mut received_pings: HashMap<SocketAddr, Instant> = HashMap::new();

    loop {
        match rx.recv() {
            Ok(val) => match val {
                PlayersStateMessage::PlayerInput(inp) => {
                    inputs.insert(inp.player_socket, inp.input);
                }
                PlayersStateMessage::GetGameId(response_sender) => {
                    let resp: Option<(u32, Vec<(SocketAddr, PlayerInput, bool)>)> = match boards_to_update.pop_front() {
                        None => None,
                        Some(board_id) => {
                            let sock_vec = boards.get_mut(&board_id).expect("Must be present").clone();
                            let new_vec: Vec<(SocketAddr, PlayerInput, bool)> = sock_vec.into_iter()
                                .filter_map(|(addr, is_left)| match inputs.get(&addr) {
                                    None => None,
                                    Some(&i) => Some((addr, i.clone(), is_left))
                                })
                                .collect();
                            if new_vec.is_empty() {
                                boards.remove(&board_id);
                            }
                            else {
                                boards_to_update.push_back(board_id);
                            }
                            Some((board_id, new_vec))
                        }
                    };

                    match response_sender.send(resp) {
                        Ok(_) => {}
                        Err(e) => error!("Cannot send response: {}", e)
                    };
                }
                PlayersStateMessage::PlayerStateMonitor => {
                    info!("Player state monitor, inputs: {}, boards: {}, board_queue: {}, waiting_boards_len: {}",
                        inputs.len(), boards.len(), boards_to_update.len(), waiting_boards.len());
                    info!("Ping monitor, received ping len: {}", received_pings.len());
                }
                PlayersStateMessage::CheckPings => {
                    received_pings = received_pings.iter()
                        .filter_map(|(&addr, &t)| if t.elapsed() > TIME_TO_REMOVE_PING {
                            debug!("Removing player_state input");
                            inputs.remove(&addr);
                            waiting_boards = waiting_boards.iter().filter_map(|(&k, &(old_addr, b))| (addr != old_addr).then_some((k, (old_addr, b)))).collect();
                            // waiting_board_id = None;
                            None
                        } else { Some((addr, t)) })
                        .collect();
                }
                PlayersStateMessage::PingReceived(peer_addr, room_choice) => {
                    match received_pings.insert(peer_addr, Instant::now()) {
                        None => {
                            if !waiting_boards.contains_key(&room_choice) {
                                debug!("Inserting player_state board, room id {}", room_choice);
                                let new_board_id: u32 = random();
                                let left_or_right: bool = random();
                                boards.insert(new_board_id, vec![(peer_addr, left_or_right)]);
                                boards_to_update.push_back(new_board_id);
                                waiting_boards.insert(room_choice, (peer_addr, new_board_id));
                            }
                            else {
                                debug!("Joining existing room id {}", room_choice);
                                let (_, board_id) = waiting_boards.get(&room_choice).unwrap();
                                let sockets_vec = match boards.get_mut(&board_id) {
                                    None => panic!("Should not happen"),
                                    Some(v) => v
                                };
                                let is_host_left = sockets_vec.first().expect("Host must exist").1;
                                sockets_vec.push((peer_addr, !is_host_left));
                                waiting_boards.remove(&room_choice);
                            };

                            if !inputs.contains_key(&peer_addr) {
                                inputs.insert(peer_addr, PlayerInput {vec_x: 0.0, vec_y: 0.0});
                            }
                        }
                        Some(_) => {}
                    };
                    let bytes = game_packet::pong_message();
                    match socket.send_to(&bytes, peer_addr) {
                        Ok(_) => {}
                        Err(e) => error!("Cannot send pong message, error: {}", e)
                    };
                }
            }
            Err(err) => {
                error!("Input error {}", err);
            }
        }
    }
}
