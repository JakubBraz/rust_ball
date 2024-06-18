use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, RecvError, Sender, SendError};
use rand::random;


pub enum PlayerGame {
    NewPlayer,
    GetSocket(u32),
    SetSocket(u32, SocketAddr),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PlayerInput {
    pub vec_x: f32,
    pub vec_y: f32,
}

#[derive(Debug)]
pub struct PlayerMessage {
    // todo socket addr as player id, is it a good idea?
    pub player_id: SocketAddr,
    pub input: PlayerInput,
}

#[derive(Debug)]
pub enum PlayersStateMessage {
    PlayerInput(PlayerMessage),
    GetGameId(Sender<Option<(u32, Option<(SocketAddr, PlayerInput)>, Option<(SocketAddr, PlayerInput)>)>>),
    AddPlayer(SocketAddr),
    RemovePlayer(SocketAddr),
}

pub fn handle_players_state(rx: Receiver<PlayersStateMessage>) {
    // todo it is overcomplicated
    let mut inputs: HashMap<SocketAddr, PlayerInput> = HashMap::new();

    let mut boards: HashMap<u32, (Option<SocketAddr>, Option<SocketAddr>)> = HashMap::new();
    let mut waiting_board_id: Option<u32> = None;
    let mut boards_to_update: VecDeque<u32> = VecDeque::new();

    loop {
        match rx.recv() {
            Ok(val) => {
                match val {
                    PlayersStateMessage::PlayerInput(inp) => {
                        println!("Player input set");
                        match inputs.insert(inp.player_id, inp.input) {
                            None => {
                                match waiting_board_id {
                                    None => {
                                        let new_board_id: u32 = random();
                                        let left_or_right: bool = random();
                                        // let left_or_right: bool = true;
                                        match left_or_right {
                                            true => boards.insert(new_board_id, (Some(inp.player_id), None)),
                                            false => boards.insert(new_board_id, (None, Some(inp.player_id)))
                                        };
                                        boards_to_update.push_back(new_board_id);
                                    }
                                    Some(board_id) => {
                                        let sockets = match boards.get(&board_id) {
                                            None => panic!("Should not happen"),
                                            Some(v) => v
                                        };
                                        let new_sockets = match sockets.0 {
                                            None => (Some(inp.player_id), sockets.1),
                                            Some(s) => (Some(s), Some(inp.player_id))
                                        };
                                        boards.insert(board_id, new_sockets);
                                        waiting_board_id = None;
                                    }
                                };
                            }
                            Some(_) => {
                                println!("Input updated");
                            }
                        };
                    }
                    PlayersStateMessage::AddPlayer(_) => {todo!("implement")}
                    PlayersStateMessage::RemovePlayer(s) => {
                        inputs.remove(&s);
                    }
                    PlayersStateMessage::GetGameId(response_sender) => {
                        let resp: Option<(u32, Option<(SocketAddr, PlayerInput)>, Option<(SocketAddr, PlayerInput)>)> = match boards_to_update.pop_front() {
                            None => None,
                            Some(board_id) => {
                                boards_to_update.push_back(board_id);
                                let (s1, s2) = boards.get(&board_id).expect("Must be present").clone();
                                Some((board_id,
                                      match s1 {
                                          None => None,
                                          Some(s) => Some((s, inputs.get(&s).expect("Must be here").clone()))
                                      },
                                      match s2 {
                                          None => None,
                                          Some(s) => Some((s, inputs.get(&s).expect("Must be here").clone()))
                                      }))
                            }
                        };

                        match response_sender.send(resp) {
                            Ok(_) => {}
                            Err(e) => println!("Cannot send response: {}", e)
                        };
                    }
                }
            }
            Err(err) => {
                println!("Input error {}", err);
            }
        }
    }
}
