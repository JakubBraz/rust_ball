use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, RecvError, Sender, SendError};
use rand::random;


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
    AddPlayer(SocketAddr),
    RemovePlayer(SocketAddr),
    PlayerStateMonitor,
}

pub fn handle_players_state(rx: Receiver<PlayersStateMessage>) {
    // todo it is overcomplicated
    let mut inputs: HashMap<SocketAddr, PlayerInput> = HashMap::new();

    let mut boards: HashMap<u32, Vec<(SocketAddr, bool)>> = HashMap::new();
    let mut waiting_board_id: Option<u32> = None;
    let mut boards_to_update: VecDeque<u32> = VecDeque::new();

    loop {
        match rx.recv() {
            Ok(val) => {
                match val {
                    PlayersStateMessage::PlayerInput(inp) => {
                        // println!("Player input set");
                        match inputs.insert(inp.player_socket, inp.input) {
                            None => {
                                match waiting_board_id {
                                    None => {
                                        let new_board_id: u32 = random();
                                        let left_or_right: bool = random();
                                        // let left_or_right: bool = true;
                                        boards.insert(new_board_id, vec![(inp.player_socket, left_or_right)]);
                                        boards_to_update.push_back(new_board_id);
                                        waiting_board_id = Some(new_board_id);
                                    }
                                    Some(board_id) => {
                                        let sockets_vec = match boards.get_mut(&board_id) {
                                            None => panic!("Should not happen"),
                                            Some(v) => v
                                        };
                                        let is_host_left = sockets_vec.first().expect("Host must exist").1;
                                        sockets_vec.push((inp.player_socket, !is_host_left));
                                        waiting_board_id = None;
                                    }
                                };
                            }
                            Some(_) => {
                                // println!("Input updated");
                            }
                        };
                    }
                    PlayersStateMessage::AddPlayer(_) => {todo!("implement")}
                    PlayersStateMessage::RemovePlayer(s) => {
                        inputs.remove(&s);
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
                            Err(e) => println!("Cannot send response: {}", e)
                        };
                    }
                    PlayersStateMessage::PlayerStateMonitor => {
                        println!("Player state monitor, inputs: {}, boards: {}, board_queue: {}", inputs.len(), boards.len(), boards_to_update.len());
                    }
                }
            }
            Err(err) => {
                println!("Input error {}", err);
            }
        }
    }
}
