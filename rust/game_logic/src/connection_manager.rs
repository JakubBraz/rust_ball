use std::net::SocketAddr;
use std::process::id;
use std::sync::mpsc::{channel, RecvError, Sender, SendError};
use std::thread::current;
use std::time::Duration;
use log::{debug, error, info, warn};

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::players_state::PlayersStateMessage;

const CLIENT_MESSAGE_LEN: usize = 2;

pub async fn handle_connections(sender: Sender<PlayersStateMessage>) {
    let listener = TcpListener::bind("0.0.0.0:8018").await.unwrap();
    loop {
        match listener.accept().await {
            Ok((mut stream, addr)) => {
                info!("New TCP connection from {}", addr);
                let sender_clone = sender.clone();
                tokio::spawn(async move {
                    handle_socket(stream, addr, sender_clone).await;
                });
            }
            Err(e) => error!("Error listening on TCP socket, error: {}", e)
        };
    }
}

async fn handle_socket(mut stream: TcpStream, peer_addr: SocketAddr, sender: Sender<PlayersStateMessage>) {
    let mut buff = [0; CLIENT_MESSAGE_LEN];
    let mut room_id: Option<u16> = None;
    let (tx, rx) = channel();

    let mut player_id = None;

    loop {
        debug!("Reading tcp bytes...");
        match stream.read(&mut buff).await {
            Ok(byte_len) => match room_id {
                None => {
                    debug!("{} bytes read: {:?}", byte_len, &buff[0..byte_len]);
                    if byte_len < CLIENT_MESSAGE_LEN {
                        info!("Disconnecting {}", peer_addr);
                        send_remove_player(&sender, player_id);
                        return;
                    } else {
                        let requested_room = u16::from_ne_bytes(buff);
                        room_id = Some(requested_room);
                        match sender.send(PlayersStateMessage::AddPlayer((tx.clone(), requested_room))) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Cannot send AddPlayer message, error: {}", e);
                                return;
                            }
                        };

                        player_id = match rx.recv() {
                            Ok(val) => Some(val),
                            Err(e) => {
                                error!("Cannot receive player id, error: {}", e);
                                return;
                            }
                        };

                        let p_id: u64 = player_id.unwrap();

                        match stream.write_all(&p_id.to_ne_bytes()).await {
                            Ok(_) => {
                                match stream.flush().await {
                                    Ok(_) => debug!("Player ID {} sent via TCP", p_id),
                                    Err(e) => {
                                        error!("Cannot flush TCP stream, error: {}", e);
                                        send_remove_player(&sender, player_id);
                                        return;
                                    }
                                };
                            }
                            Err(e) => {
                                error!("Cannot write TCP bytes, error: {}", e);
                                send_remove_player(&sender, player_id);
                                return;
                            }
                        };
                    }
                }
                Some(val) => {
                    warn!("Unexpected TCP message: {:?}, room_id already received: {}", &buff[0..byte_len], val);
                }
            }
            Err(e) => {
                error!("TCP read error: {}", e);
                info!("Disconnecting {}", peer_addr);
                send_remove_player(&sender, player_id);
                return;
            }
        }
    }
}

fn send_remove_player(sender: &Sender<PlayersStateMessage>, player_id: Option<u64>) {
    let player_id = player_id.unwrap_or_else(|| {
        warn!("No player id provided");
        0
    });
    match sender.send(PlayersStateMessage::RemovePlayer(player_id)) {
        Ok(_) => {}
        Err(e) => error!("Cannot send RemovePlayer message, error: {}", e)
    };
}
