use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, RecvError, Sender, SendError};
use std::time::{Duration, Instant};
use log::{debug, error, info};
use crate::game_packet;
use crate::players_state::PlayersStateMessage;

const TIME_TO_REMOVE: Duration = Duration::from_secs(30);

pub enum PingMessage {
    CheckPings,
    PingReceived(SocketAddr),
    PingStateMonitor,
}

pub fn handle_ping(receiver: Receiver<PingMessage>, player_state_sender: Sender<PlayersStateMessage>, socket: UdpSocket) {
    //todo it would be better to move all that logic into "player_state" module
    let mut received_pings: HashMap<SocketAddr, Instant> = HashMap::new();

    loop {
        match receiver.recv() {
            Ok(message) => match message {
                PingMessage::CheckPings => {
                    received_pings = received_pings.iter()
                        .filter_map(|(&addr, &t)| if t.elapsed() > TIME_TO_REMOVE {
                            match player_state_sender.send(PlayersStateMessage::RemovePlayer(addr)) {
                                Ok(_) => {}
                                Err(e) => error!("Cannot send remove player, error: {:?}", e)
                            };
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
                        Err(e) => error!("Cannot send pong message, error: {}", e)
                    };
                }
                PingMessage::PingStateMonitor => {
                    info!("Ping monitor, received ping len: {}", received_pings.len());
                }
            },
            Err(e) => error!("Cannot receive ping, error: {}", e)
        };
    }
}
