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
    pub player_id: u64,
    pub player_socket: SocketAddr,
    pub input: PlayerInput,
}

#[derive(Debug)]
pub enum PlayersStateMessage {
    PlayerInput(PlayerMessage),
    GetGameId(Sender<Option<(u32, Vec<(Option<SocketAddr>, PlayerInput, bool)>)>>),
    PlayerStateMonitor,
    AddPlayer((Sender<u64>, u16)),
    RemovePlayer(u64),
}

pub fn handle_players_state(rx: Receiver<PlayersStateMessage>, socket: UdpSocket) {
    let mut player_info: HashMap<u64, (bool, u32, PlayerInput, Option<SocketAddr>)> = HashMap::new();
    let mut boards_to_update: VecDeque<u32> = VecDeque::new();
    let mut board_members: HashMap<u32, Vec<u64>> = HashMap::new();
    let mut waiting_boards: HashMap<u16, u32> = HashMap::new();

    loop {
        match rx.recv() {
            Ok(val) => match val {
                PlayersStateMessage::PlayerInput(inp) => {
                    if player_info.contains_key(&inp.player_id) {
                        let &(is_left, b_id,_, _) = player_info.get(&inp.player_id).unwrap();
                        player_info.insert(inp.player_id, (is_left, b_id, inp.input, Some(inp.player_socket)));
                    }
                    else {
                        error!("Player id {} not added yet", inp.player_id);
                    }
                }
                PlayersStateMessage::GetGameId(response_sender) => {
                    //todo use iter().take_while
                    let resp: Option<(u32, Vec<(Option<SocketAddr>, PlayerInput, bool)>)> = match boards_to_update.pop_front() {
                        None => None,
                        Some(board_id) => {
                            match board_members.get(&board_id) {
                                None => None,
                                Some(members) => {
                                    boards_to_update.push_back(board_id);
                                    let sockets: Vec<(Option<SocketAddr>, PlayerInput, bool)> = members.iter().map(|id| {
                                        let &(is_left, _b_id, inp, sock) = player_info.get(id).unwrap();
                                        (sock, inp, is_left)
                                    }).collect();
                                    Some((board_id, sockets))
                                }
                            }
                        }
                    };

                    match response_sender.send(resp) {
                        Ok(_) => {}
                        Err(e) => error!("Cannot send response: {}", e)
                    };
                }
                PlayersStateMessage::PlayerStateMonitor => {
                    info!("Player state monitor, player_info: {}, board_members: {}, boards_to_update: {}, waiting_boards_len: {}",
                        player_info.len(), board_members.len(), boards_to_update.len(), waiting_boards.len());
                }
                PlayersStateMessage::AddPlayer((ch, room_id)) => {
                    let player_id: u64 = random();
                    match ch.send(player_id) {
                        Ok(_) => {}
                        Err(e) => error!("Cannot send player id, error: {}", e)
                    };
                    if waiting_boards.contains_key(&room_id) {
                        let r_id = waiting_boards.get(&room_id).unwrap();
                        let mut players = board_members.get_mut(r_id).unwrap();
                        let host = players.first().unwrap();
                        let &(host_left, b_id, inp, _) = player_info.get(host).unwrap();
                        players.push(player_id);
                        player_info.insert(player_id, (!host_left, b_id, PlayerInput { vec_x: 0.0, vec_y: 0.0 }, None));
                        waiting_boards.remove(&room_id);
                        info!("Player {} inserted into waiting room {}", player_id, room_id);
                    }
                    else {
                        let b_id: u32 = random();
                        let left_or_right: bool = random();
                        waiting_boards.insert(room_id, b_id);
                        board_members.insert(b_id, vec![player_id]);
                        boards_to_update.push_back(b_id);
                        player_info.insert(player_id, (left_or_right, b_id, PlayerInput { vec_x: 0.0, vec_y: 0.0 }, None));
                        info!("Creating new waiting room {}", b_id);
                    }
                },
                PlayersStateMessage::RemovePlayer((player_id)) => {
                    info!("Removing player {}", player_id);
                    if player_info.contains_key(&player_id) {
                        let &(_, b_id, _, _) = player_info.get(&player_id).unwrap();
                        player_info.remove(&player_id);
                        let members = board_members.get(&b_id).unwrap().clone();
                        let members: Vec<u64> = members.into_iter().filter(|&x| x != player_id).collect();
                        if !members.is_empty() {
                            board_members.insert(b_id, members);
                        }
                        else {
                            board_members.remove(&b_id);
                        }
                        waiting_boards = waiting_boards.into_iter().filter(|&(k, v)| v != b_id ).collect();
                    }
                },

                // PlayersStateMessage::CheckPings => {
                //     received_pings = received_pings.iter()
                //         .filter_map(|(&addr, &t)| if t.elapsed() > TIME_TO_REMOVE_PING {
                //             debug!("Removing player_state input");
                //             inputs.remove(&addr);
                //             waiting_boards = waiting_boards.iter().filter_map(|(&k, &(old_addr, b))| (addr != old_addr).then_some((k, (old_addr, b)))).collect();
                //             // waiting_board_id = None;
                //             None
                //         } else { Some((addr, t)) })
                //         .collect();
                // }
                // PlayersStateMessage::PingReceived(peer_addr, room_choice) => {
                //     match received_pings.insert(peer_addr, Instant::now()) {
                //         None => {
                //             if !waiting_boards.contains_key(&room_choice) {
                //                 debug!("Inserting player_state board, room id {}", room_choice);
                //                 let new_board_id: u32 = random();
                //                 let left_or_right: bool = random();
                //                 boards.insert(new_board_id, vec![(peer_addr, left_or_right)]);
                //                 boards_to_update.push_back(new_board_id);
                //                 waiting_boards.insert(room_choice, (peer_addr, new_board_id));
                //             }
                //             else {
                //                 debug!("Joining existing room id {}", room_choice);
                //                 let (_, board_id) = waiting_boards.get(&room_choice).unwrap();
                //                 let sockets_vec = match boards.get_mut(&board_id) {
                //                     None => panic!("Should not happen"),
                //                     Some(v) => v
                //                 };
                //                 let is_host_left = sockets_vec.first().expect("Host must exist").1;
                //                 sockets_vec.push((peer_addr, !is_host_left));
                //                 waiting_boards.remove(&room_choice);
                //             };
                //
                //             if !inputs.contains_key(&peer_addr) {
                //                 inputs.insert(peer_addr, PlayerInput {vec_x: 0.0, vec_y: 0.0});
                //             }
                //         }
                //         Some(_) => {}
                //     };
                // }
            }
            Err(err) => {
                error!("Input error {}", err);
            }
        }
    }
}
