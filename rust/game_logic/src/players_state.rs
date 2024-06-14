use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, RecvError, Sender};


pub enum PlayerGame {
    NewPlayer,
    GetSocket(u32),
    SetSocket(u32, SocketAddr),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PlayerInput {
    pub start_x: f32,
    pub start_y: f32,
    pub current_x: f32,
    pub current_y: f32,
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
    //todo is a good idea to send the whole HashMap?
    GetAllActiveInput(Sender<HashMap<SocketAddr, PlayerInput>>),
    AddPlayer(SocketAddr),
    RemovePlayer(SocketAddr),
}

pub fn handle_players_state(rx: Receiver<PlayersStateMessage>) {
    let mut inputs: HashMap<SocketAddr, PlayerInput> = HashMap::new();
    let mut playerSocket: HashMap<u32, SocketAddr> = HashMap::new();
    let mut gamePlayers: HashMap<u32, (u32, u32)> = HashMap::new();

    loop {
        match rx.recv() {
            Ok(val) => {
                match val {
                    PlayersStateMessage::PlayerInput(inp) => {
                        println!("Player input set");
                        inputs.insert(inp.player_id, inp.input);
                    }
                    PlayersStateMessage::AddPlayer(_) => {todo!("implement")}
                    PlayersStateMessage::RemovePlayer(s) => {
                        inputs.remove(&s);
                    }
                    PlayersStateMessage::GetAllActiveInput(response_sender) => {
                        match response_sender.send(inputs.clone()) {
                            Ok(()) => {
                                // println!("Sending {} inputs", inputs.len());
                            }
                            Err(err) => {
                                println!("Cannot send input: {}", err);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!("Input error {}", err);
            }
        }
    }
}
