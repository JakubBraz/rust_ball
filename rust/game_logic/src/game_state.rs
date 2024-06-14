use std::env;
use std::net::UdpSocket;
use std::sync::mpsc::{channel, Sender};
use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::physics::GamePhysics;
use crate::players_state::{PlayerInput, PlayersStateMessage};
// use crate::PlayerInput;

#[derive(Default, PartialEq)]
pub struct GameState {
    pub player_x: f32,
    pub player_y: f32,
    pub player_r: f32,
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_r: f32,
    pub touch_vec_x: f32,
    pub touch_vec_y: f32,
}

impl GameState {
    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.player_x.to_ne_bytes(),
            self.player_y.to_ne_bytes(),
            self.player_r.to_ne_bytes(),
            self.ball_x.to_ne_bytes(),
            self.ball_y.to_ne_bytes(),
            self.ball_r.to_ne_bytes(),
            // todo don't send to client touch_vec, let it handle drawing it on its own
            self.touch_vec_x.to_ne_bytes(),
            self.touch_vec_y.to_ne_bytes(),
        ].concat()
    }
}

pub fn handle_game_state(send_to_input: Sender<PlayersStateMessage>, socket: UdpSocket, ) {
    let (tx, rx) = channel();

    let step: Duration = Duration::from_secs_f32(1.0 / 60.0);
    //todo read scaling from client socket
    let scaling: f32 = 20.0;

    let mut game_physics = GamePhysics::init();

    let mut last_update = Duration::from_millis(0);
    let game_duration = Instant::now();

    let mut prev_game_state = Default::default();

    loop {
        let i = Instant::now();
        send_to_input.send(PlayersStateMessage::GetAllActiveInput(tx.clone())).unwrap();
        let player_input = rx.recv().unwrap();

        // todo pick game state depending on player id (socket addr)
        for (addr, inp) in player_input {

            // println!("player input {:?}", inp);
            if inp == PlayerInput::default() {
                game_physics.reset_kick();
            }
            game_physics.move_mouse((inp.start_x, inp.start_y), (inp.current_x, inp.current_y), scaling);

            // todo 15ms passes for this loop, is it enough sufficient?
            while game_duration.elapsed() - last_update >= step {
                // println!("{:?} STEP", game_duration.elapsed());
                //todo count physics stepped performed to measure if it happens every 1/60 sec
                game_physics.step();
                last_update += step;
            }

            // todo sending response in handling physics? is it a good idea? maybe move it to another thread?
            let game_state = game_physics.get_game_state();
            if prev_game_state != game_state {
                let bytes = game_state.to_bytes();

                if env::consts::OS == "windows" {
                    //todo temporary windows workaround
                    let s = UdpSocket::bind("127.0.0.1:8053").unwrap();
                    s.connect(addr).unwrap();
                    s.send(&bytes).unwrap();
                }
                else {
                    match socket.send_to(&bytes, addr) {
                        Ok(b) => {
                            // println!("{} bytes sent", b);
                        }
                        Err(e) => {
                            println!("Cannot send, {}", e);
                            panic!("Cannot send to socket!");
                        }
                    }
                }

                prev_game_state = game_state;
            }
        }

        //todo why this sleep is necessary? shouldnt prev_game_state != game_state be enough?
        sleep(Duration::from_millis(30));
        // println!("Physics loop took: {:?}", i.elapsed());
    }
}
