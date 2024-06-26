use std::sync::mpsc::{Sender, SendError};
use std::thread::sleep;
use std::time::Duration;
use crate::ping_handler::PingMessage;

pub fn handle_timer(ping_sender: Sender<PingMessage>) {
    let sleep_time = Duration::from_millis(1000);

    loop {
        match ping_sender.send(PingMessage::CheckPings) {
            Ok(_) => {}
            Err(e) => println!("Cannot send pong message, error: {}", e)
        };
        sleep(sleep_time);
    }
}
