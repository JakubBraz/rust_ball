use std::cmp::PartialEq;
use std::collections::HashMap;
use std::io::ErrorKind::TimedOut;
use std::net::{SocketAddr, UdpSocket};
use std::process::{exit, id};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, SendError};
use std::thread::{current, sleep, spawn};
use std::time::{Duration, Instant};
use rapier2d::counters::Timer;
use game_logic::physics::GamePhysics;

#[derive(Debug, Default, Clone, PartialEq)]
struct PlayerInput {
    start_x: f32,
    start_y: f32,
    current_x: f32,
    current_y: f32,
}

#[derive(Debug)]
struct PlayerMessage {
    // todo socket addr as player id, is it a good idea?
    player_id: SocketAddr,
    input: PlayerInput,
}

#[derive(Debug)]
enum InputRequest {
    PlayerInput(PlayerMessage),
    //todo is a good idea to send the whole HashMap?
    GetAllActiveInput(Sender<HashMap<SocketAddr, PlayerInput>>),
    AddPlayer(SocketAddr),
    RemovePlayer(SocketAddr),
}

fn main() {
    let (tx_input, rx_input) = channel();

    let mut socket = UdpSocket::bind("0.0.0.0:8019").expect("Cannot create socket");

    let input_handler = spawn(|| handle_input(rx_input));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let socket_handler = spawn(|| handle_socket(tx_clone, socket_clone));
    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx_input.clone();
    let game_handler = spawn(|| handle_game_state(tx_clone, socket_clone));

    input_handler.join();
}

fn handle_game_state(send_to_input: Sender<InputRequest>, socket: UdpSocket, ) {
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
        send_to_input.send(InputRequest::GetAllActiveInput(tx.clone())).unwrap();
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

                match socket.send_to(&bytes, addr) {
                    Ok(b) => {
                        // println!("{} bytes sent", b);
                    }
                    Err(e) => {
                        println!("Cannot send, {}", e);
                    }
                }
                prev_game_state = game_state;
            }
        }

        //todo why this sleep is necessary? shouldnt prev_game_state != game_state be enough?
        sleep(Duration::from_millis(30));
        println!("Physics loop took: {:?}", i.elapsed());
    }
}

fn handle_input(rx: Receiver<InputRequest>) {
    let mut inputs: HashMap<SocketAddr, PlayerInput> = HashMap::new();
    // inputs.insert(1, Default::default());
    loop {
        match rx.recv() {
            Ok(val) => {
                match val {
                    InputRequest::PlayerInput(inp) => {
                        println!("Player input set");
                        inputs.insert(inp.player_id, inp.input);
                    }
                    InputRequest::AddPlayer(_) => {todo!("implement")}
                    InputRequest::RemovePlayer(_) => {todo!("implement removing playe")}
                    InputRequest::GetAllActiveInput(response_sender) => {
                        match response_sender.send(inputs.clone()) {
                            Ok(()) => {
                                // println!("Sending {} inputs", inputs.len());
                            }
                            Err(err) => {
                                println!("Cannot send input: {}", err);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!("Input error {}", err);
            }
        }
    }
}

fn handle_socket(input_sender: Sender<InputRequest>, socket: UdpSocket) {
    let mut buf = [0; 128];

    println!("Waiting for 'hello' packet");
    let (_, client_address) = socket.recv_from(&mut buf).expect("Cannot read hello msg");

    loop {
        println!("Reading socket...");
        match socket.recv_from(&mut buf) {
            Ok((len, client_address)) => {
                let i = Instant::now();
                let bytes = &buf[0..len];
                let start_x = i16::from_ne_bytes(bytes[0..2].try_into().unwrap()) as f32;
                let start_y = i16::from_ne_bytes(bytes[2..4].try_into().unwrap()) as f32;
                let current_x = i16::from_ne_bytes(bytes[4..6].try_into().unwrap()) as f32;
                let current_y = i16::from_ne_bytes(bytes[6..8].try_into().unwrap()) as f32;
                // todo make client dont send touch vector always it changes; instead, send it only it  exceeds some threshold (for example 8 possible "speeds")
                println!("{} bytes read, message: {:?}, {}, {}, {} {}", len, bytes, start_x, start_y, current_x, current_y);

                let player_id = client_address;
                match input_sender.send(InputRequest::PlayerInput(PlayerMessage {player_id,  input: PlayerInput{ start_x, start_y, current_x, current_y }})) {
                    Ok(val) => {}
                    Err(err) => {
                        println!("Cannot send: {}", err);
                    }
                }
                println!("handling_socket takes {:?}", i.elapsed());
            }
            Err(e) => {
                println!("Error read: {}", e);
                panic!("error");
            }
        }
    }
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

                    game_physics.move_mouse((start_x, start_y), (current_x, current_y), scaling);
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
