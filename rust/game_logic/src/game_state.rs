use std::collections::{HashMap, VecDeque};
use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{channel, Sender};
use std::thread::sleep;
use std::time::{Duration, Instant};
use log::{debug, error};
use crate::game_packet;
use crate::physics::GamePhysics;
use crate::players_state::{PlayerInput, PlayersStateMessage};
// use crate::PlayerInput;

#[derive(Default, PartialEq)]
pub struct GameState {
    pub is_left: bool,
    pub ball_x: f32,
    pub ball_y: f32,
    pub player_1_1_x: f32,
    pub player_1_1_y: f32,
    pub player_1_2_x: f32,
    pub player_1_2_y: f32,
    pub player_2_1_x: f32,
    pub player_2_1_y: f32,
    pub player_2_2_x: f32,
    pub player_2_2_y: f32,
}

pub fn handle_game_state(send_to_input: Sender<PlayersStateMessage>, socket: UdpSocket, ) {
    let (tx, rx) = channel();

    let step: Duration = Duration::from_secs_f32(1.0 / 60.0);
    let game_duration = Instant::now();

    let mut boards: HashMap<u32, (GamePhysics, Duration)> = HashMap::new();

    loop {
        let i = Instant::now();
        send_to_input.send(PlayersStateMessage::GetGameId(tx.clone())).unwrap();
        match rx.recv().unwrap() {
            None => {
                // println!("Nothing to update");
                sleep(Duration::from_millis(500));
            }
            Some((board_id, players_vec)) => {
                match boards.get_mut(&board_id) {
                    None => {
                        debug!("Creating new game_state board");
                        let mut g = GamePhysics::init();
                        g.add_player1();
                        g.add_player2();
                        // if player_left.is_some() {
                        //     g.add_player1();
                        // }
                        // else {
                        //     g.add_player2();
                        // }
                        //todo move the state into player_state
                        boards.insert(board_id, (g, game_duration.elapsed()));
                        //todo send state to socket just after creation?
                    }
                    Some((game_physics, last_update)) => {
                        if players_vec.is_empty() {
                            debug!("Removing game_state board");
                            boards.remove(&board_id);
                        }
                        else {
                            //todo updating game state should be done after sending inputs (but maybe it doesnt matter too much)
                            let mut physics_updated = false;
                            while game_duration.elapsed() - *last_update >= step {
                                // debug!("{:?} STEP", game_duration.elapsed());
                                //todo count physics stepped performed to measure if it happens every 1/60 sec
                                game_physics.step();
                                *last_update += step;
                                physics_updated = true;
                            }
                            // debug!("After while");

                            for (i, &(addr, inp, is_left)) in players_vec.iter().enumerate() {
                                //todo use index 1, 2 for left player and 3, 4 for the right one
                                game_physics.move_mouse(if is_left { 0 } else { 1 }, inp.vec_x, inp.vec_y);
                                if physics_updated {
                                    match addr {
                                        // None => debug!("Before input, no socket to send"),
                                        None => {},
                                        Some(val) => send_game_state(is_left, val, &socket, game_physics)
                                    }
                                }
                            }
                        }
                    }
                };
            }
        };

        //todo why this sleep is necessary? shouldnt prev_game_state != game_state be enough?
        // sleep(Duration::from_millis(30));
        // println!("Physics loop took: {:?}", i.elapsed());
    }
}

fn send_game_state(is_left: bool, addr: SocketAddr, socket: &UdpSocket, game_physics: &mut GamePhysics) {
    //todo use index 1, 2 for left player and 3, 4 for the right one
    // game_physics.move_mouse(if is_left { 0 } else { 1 }, inp.vec_x, inp.vec_y);

    // todo sending response in handling physics? is it a good idea? maybe move it to another thread?
    let (ball_x, ball_y, player_x, player_y, player2_x, player2_y) = game_physics.get_game_state();
    let mut current_game_state = GameState::default();
    current_game_state.is_left = is_left;
    current_game_state.ball_x = ball_x;
    current_game_state.ball_y = ball_y;
    current_game_state.player_1_1_x = player_x;
    current_game_state.player_1_1_y = player_y;
    current_game_state.player_2_1_x = player2_x;
    current_game_state.player_2_1_y = player2_y;

    let bytes = game_packet::to_bytes(&current_game_state, 0);

    match socket.send_to(&bytes, addr) {
        Ok(b) => {
            // println!("{} bytes sent", b);
        }
        Err(e) => {
            error!("Cannot send, {}", e);
            panic!("Cannot send to socket!");
        }
    }
}
