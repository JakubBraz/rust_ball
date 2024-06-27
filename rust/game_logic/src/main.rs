pub mod game_packet;
mod players_state;

mod physics;
mod game_state;
mod timer;
mod ping_handler;

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
    let (tx_ping_pong, rx_ping_pong) = channel();

    let mut socket = UdpSocket::bind("0.0.0.0:8019").expect("Cannot create socket");

    let input_handler = spawn(|| handle_players_state(rx_input));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let ping_pong = tx_ping_pong.clone();
    let socket_handler = spawn(|| handle_socket(tx_clone, ping_pong, socket_clone));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let game_handler = spawn(|| handle_game_state(tx_clone, socket_clone));

    let ping_pong = tx_ping_pong.clone();
    let tx_clone = tx_input.clone();
    let timer_handler = spawn(|| timer::handle_timer(ping_pong, tx_clone));

    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let ping_handler = spawn(|| ping_handler::handle_ping(rx_ping_pong, tx_clone, socket_clone));

    input_handler.join();
}
