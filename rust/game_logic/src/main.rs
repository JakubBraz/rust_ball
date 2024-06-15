pub mod game_packet;
mod players_state;

mod physics;
mod game_state;

use std::cmp::PartialEq;
use std::collections::HashMap;
use std::io::ErrorKind::TimedOut;
use std::net::{SocketAddr, UdpSocket};
use std::process::{exit, id};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, SendError};
use std::thread::{current, sleep, spawn};
use std::time::{Duration, Instant};
use rapier2d::counters::Timer;
use crate::game_packet::handle_socket;
use crate::game_state::handle_game_state;
use crate::physics::GamePhysics;
use crate::players_state::{handle_players_state, PlayersStateMessage};

fn main() {
    let (tx_input, rx_input) = channel();

    let mut socket = UdpSocket::bind("0.0.0.0:8019").expect("Cannot create socket");

    let input_handler = spawn(|| handle_players_state(rx_input));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let socket_handler = spawn(|| handle_socket(tx_clone, socket_clone));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let game_handler = spawn(|| handle_game_state(tx_clone, socket_clone));

    input_handler.join();
}

fn old_main() {
    println!("test");

    loop {
        let step: Duration = Duration::from_secs_f32(1.0 / 60.0);
        //todo read scaling from client socket
        let scaling: f32 = 20.0;

        let mut buf = [0; 128];
        let mut socket = UdpSocket::bind("0.0.0.0:8019").expect("Cannot create socket");

        println!("Waiting for 'hello' packet");
        let (_, client_address) = socket.recv_from(&mut buf).expect("Cannot read hello msg");

        // todo this socket read takes 16ms, it is too long, rewrite to threads? async?
        // socket.set_read_timeout(Some(Duration::from_millis(1))).expect("Cannot set timeout");
        socket.set_read_timeout(Some(Duration::from_nanos(1))).expect("Cannot set timeout");

        let mut game_physics = GamePhysics::init();

        let mut last_update = Duration::from_millis(0);
        let game_duration = Instant::now();
        loop {
            // println!("loop start");
            let i = Instant::now();

            match socket.recv(&mut buf) {
                Ok(len) => {
                    let bytes = &buf[0..len];
                    // let s = String::from_utf8_lossy(bytes);
                    // println!("{} bytes read, message: {:?}, {}, {} {}", len, bytes, s, x, y);
                    let start_x = i16::from_ne_bytes(bytes[0..2].try_into().unwrap()) as f32;
                    let start_y = i16::from_ne_bytes(bytes[2..4].try_into().unwrap()) as f32;
                    let current_x = i16::from_ne_bytes(bytes[4..6].try_into().unwrap()) as f32;
                    let current_y = i16::from_ne_bytes(bytes[6..8].try_into().unwrap()) as f32;
                    println!("{} bytes read, message: {:?}, {}, {}, {} {}", len, bytes, start_x, start_y, current_x, current_y);

                    game_physics.move_mouse(0.0, 0.0);
                }
                Err(e) => {
                    if e.kind() != TimedOut {
                        println!("Error read: {}", e);
                        break
                    }
                }
            }

            // println!("socket rev takes {:?}", i.elapsed());

            // let elapsed = game_duration.elapsed();
            // todo 15ms passes for this loop, is it enough sufficient?
            while game_duration.elapsed() - last_update >= step {
                // println!("{:?} STEP", game_duration.elapsed());
                //todo count physics stepped performed to measure if it happens every 1/60 sec
                game_physics.step();
                last_update += step;

                let (player_x, player_y, player_r, touch_vec_x, touch_vec_y) = game_physics.player3();
                let (_, _, _, ball_x, ball_y, ball_r) = game_physics.player();
                let bytes = [
                    player_x.to_ne_bytes(),
                    player_y.to_ne_bytes(),
                    player_r.to_ne_bytes(),
                    ball_x.to_ne_bytes(),
                    ball_y.to_ne_bytes(),
                    ball_r.to_ne_bytes(),
                    touch_vec_x.to_ne_bytes(),
                    touch_vec_y.to_ne_bytes(),
                ].concat();
                match socket.send_to(&bytes, client_address) {
                    Ok(b) => {
                        println!("{} bytes sent", b);
                    }
                    Err(e) => {
                        println!("Cannot send, {}", e);
                    }
                }
            }
            println!("WHOLE LOOP takes {:?}", i.elapsed());
        }
    }
}
