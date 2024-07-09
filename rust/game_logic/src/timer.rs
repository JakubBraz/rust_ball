use std::sync::mpsc::{Sender, SendError};
use std::thread::sleep;
use std::time::{Duration, Instant};
use log::error;
use crate::players_state::PlayersStateMessage;

pub fn handle_timer(player_state_sender: Sender<PlayersStateMessage>) {
    let sleep_time = Duration::from_millis(1000);
    let monitor_time = Duration::from_secs(60);
    let mut monitor_timer = Instant::now();

    loop {
        if monitor_timer.elapsed() > monitor_time {
            monitor_timer = Instant::now();
            match player_state_sender.send(PlayersStateMessage::PlayerStateMonitor) {
                Ok(_) => {}
                Err(e) => error!("Cannot send player monitor, error: {}", e)
            }
        }

        sleep(sleep_time);
    }
}
