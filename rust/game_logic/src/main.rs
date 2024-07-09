pub mod game_packet;
mod players_state;

mod physics;
mod game_state;
mod timer;
mod connection_manager;

use std::cmp::PartialEq;
use std::collections::HashMap;
use std::io::ErrorKind::TimedOut;
use std::net::{SocketAddr, UdpSocket};
use std::process::{exit, id};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, SendError};
use std::thread::{current, sleep, spawn};
use std::time::{Duration, Instant};
use rapier2d::counters::Timer;
use log::{debug, error, info, LevelFilter, log, trace, warn};
use std::io::Write;
use crate::game_packet::handle_socket;
use crate::game_state::handle_game_state;
use crate::physics::GamePhysics;
use crate::players_state::{handle_players_state, PlayersStateMessage};

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .format_timestamp_millis()
        // .format(|buff, record| writeln!(buff, "[{} {} {}]\n    {}", buff.timestamp_millis(), record.level(), record.target(), record.args()))
        .init();

    info!("Starting, process id: {}, thread id: {:?}", std::process::id(), std::thread::current().id());

    let (tx_input, rx_input) = channel();

    let mut socket = UdpSocket::bind("0.0.0.0:8019").expect("Cannot create socket");

    let tx_clone = tx_input.clone();
    spawn(|| {
        info!("Starting thread for tcp processing, thread id {:?}", current().id());
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        rt.block_on(async {
            connection_manager::handle_connections(tx_clone).await;
        });
    });

    let socket_clone = socket.try_clone().unwrap();
    let input_handler = spawn(|| handle_players_state(rx_input, socket_clone));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let socket_handler = spawn(|| handle_socket(tx_clone, socket_clone));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let game_handler = spawn(|| handle_game_state(tx_clone, socket_clone));

    let tx_clone = tx_input.clone();
    let timer_handler = spawn(|| timer::handle_timer(tx_clone));

    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();

    input_handler.join();
}
