use std::env;
use std::net::UdpSocket;
use std::sync::mpsc::{channel, Sender};
use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::game_packet;
use crate::physics::GamePhysics;
use crate::players_state::{PlayerInput, PlayersStateMessage};
// use crate::PlayerInput;

#[derive(Default, PartialEq)]
pub struct GameState {
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

    let mut game_physics = GamePhysics::init();

    let mut last_update = Duration::from_millis(0);
    let game_duration = Instant::now();

    let mut prev_game_state: GameState = Default::default();

    loop {
        let i = Instant::now();
        send_to_input.send(PlayersStateMessage::GetAllActiveInput(tx.clone())).unwrap();
        let player_input = rx.recv().unwrap();

        // todo pick game state depending on player id (socket addr)
        for (addr, inp) in player_input {
            game_physics.move_mouse(inp.vec_x, inp.vec_y);

            // todo 15ms passes for this loop, is it enough sufficient?
            while game_duration.elapsed() - last_update >= step {
                // println!("{:?} STEP", game_duration.elapsed());
                //todo count physics stepped performed to measure if it happens every 1/60 sec
                game_physics.step();
                last_update += step;
            }

            // todo sending response in handling physics? is it a good idea? maybe move it to another thread?
            let (ball_x, ball_y, player_x, player_y) = game_physics.get_game_state();
            let mut current_game_state = GameState::default();
            current_game_state.ball_x = ball_x;
            current_game_state.ball_y = ball_y;
            current_game_state.player_1_1_x = player_x;
            current_game_state.player_1_1_y = player_y;

            if prev_game_state != current_game_state {
                let bytes = game_packet::to_bytes(&current_game_state, 0);

                match socket.send_to(&bytes, addr) {
                    Ok(b) => {
                        // println!("{} bytes sent", b);
                    }
                    Err(e) => {
                        println!("Cannot send, {}", e);
                        panic!("Cannot send to socket!");
                    }
                }

                prev_game_state = current_game_state;
            }
        }

        //todo why this sleep is necessary? shouldnt prev_game_state != game_state be enough?
        sleep(Duration::from_millis(30));
        // println!("Physics loop took: {:?}", i.elapsed());
    }
}
