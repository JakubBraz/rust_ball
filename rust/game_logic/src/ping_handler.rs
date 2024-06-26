use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, RecvError};
use std::time::{Duration, Instant};
use crate::game_packet;

const TIME_TO_REMOVE: Duration = Duration::from_secs(30);

pub enum PingMessage {
    CheckPings,
    PingReceived(SocketAddr)
}

pub fn handle_ping(receiver: Receiver<PingMessage>, socket: UdpSocket) {
    let mut received_pings: HashMap<SocketAddr, Instant> = HashMap::new();

    loop {
        match receiver.recv() {
            Ok(message) => match message {
                PingMessage::CheckPings => {
                    received_pings = received_pings.iter()
                        .filter_map(|(&addr, &t)| if t.elapsed() > TIME_TO_REMOVE {
                            //todo send "player_state RemovePlayer"
                            None
                        } else { Some((addr, t)) })
                        .collect();
                }
                PingMessage::PingReceived(peer_addr) => {
                    received_pings.insert(peer_addr, Instant::now());
                    // todo if addr was not in the map invoke "player_state AddPlayer"
                    let bytes = game_packet::pong_message();
                    match socket.send_to(&bytes, peer_addr) {
                        Ok(_) => {}
                        Err(e) => println!("Cannot send pong message, error: {}", e)
                    };
                }
            },
            Err(e) => println!("Cannot receive ping, error: {}", e)
        };
    }
}
