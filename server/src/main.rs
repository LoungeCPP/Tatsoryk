#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    unsafe_code,
    dead_code,
    unused_results,
)]

pub mod message;
pub mod events;
pub mod server;

extern crate time;
extern crate websocket;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use std::env;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::vec::Vec;

use std::iter::FromIterator;

use events::Client;
use events::WebSocketEvent;

use server::listen;

/// The GameState contains the whole state of the game.
/// It consists of both players, and all the clients which are currently connected.
///
#[derive(Debug)]
struct GameState {
    players: HashMap<u32, message::Player>,
    clients: HashMap<u32, Client>,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            players: HashMap::new(),
            clients: HashMap::new(),
        }
    }
}

/// Serialize the entire game state into one json string.
///
fn serialize_state(game_state: &GameState) -> String {
    let players: Vec<message::Player> = Vec::from_iter(game_state.players
                                                                 .values()
                                                                 .map(|a| a.clone()));
    let state = message::Message::WorldState {
        player_count: players.len() as u32,
        alive_players: players,
        alive_bullets: Vec::new(),
    };
    state.to_string()
}

/// Process a simple string message from the client.
///
fn process_client_message(game_state: &mut GameState, client_id: u32, message: message::Message) {
    match message {
        message::Message::StartMoving { move_x, move_y } => {
            let player = game_state.players.get_mut(&client_id).unwrap();
            player.move_x = Some(move_x);
            player.move_y = Some(move_y);
        }
        _ => panic!("Unprocessed message! {}", message.to_string()),
    }
}

/// Process a web socket event.
///
fn process_websocket_event(game_state: &mut GameState, message: WebSocketEvent) {
    match message {
        WebSocketEvent::ClientCreated { client } => {
            let _ = game_state.players.insert(client.id.clone(),
                                              message::Player::not_moving(client.id.clone(),
                                                                          0.0,
                                                                          0.0));
            let _ = game_state.clients.insert(client.id.clone(), client);
        }
        WebSocketEvent::ClientClosed { client_id } => {
            let _ = game_state.clients.remove(&client_id);
        }
        WebSocketEvent::ClientMessage { client_id, message } => {
            process_client_message(game_state, client_id, message);
        }
    }
}

/// Tries to process every available websocket event without blocking.
///
fn process_websocket_events(game_state: &mut GameState,
                            game_messages: &std::sync::mpsc::Receiver<WebSocketEvent>) {
    loop {
        match game_messages.try_recv() {
            Ok(a) => process_websocket_event(game_state, a),
            Err(e) => {
                match e {
                    std::sync::mpsc::TryRecvError::Empty => return,
                    std::sync::mpsc::TryRecvError::Disconnected => panic!("Now I am disconnected?"),
                }
            }
        }
    }
}

/// Updates the game state in one tick.
///
fn process_game_update(game_state: &mut GameState) {
    for (_, player) in &mut game_state.players {
        player.x += player.move_x.unwrap_or(0.0);
        player.y += player.move_y.unwrap_or(0.0);
    }
}

/// Send the current, entire state to each client.
///
fn send_state_updates(game_state: &GameState) {
    let value = serialize_state(game_state);

    for (_, client) in &game_state.clients {
        // Always ignore if the send fails.
        // We will eventually get a disconnect WebSocketMessage where we will cleanly do the disconnect.
        let _ = client.send(value.clone());
    }
}

/// Runs the main game loop.
///
/// The general idea for the game loop is to update the game state every 16 milliseconds (60 FPS), processing messages along the way.
///
fn game_loop(game_messages: std::sync::mpsc::Receiver<WebSocketEvent>) {
    let mut game_state = GameState::new();

    let start_time = time::precise_time_ns();
    let mut iter: u64 = 0;
    let iter_length: u64 = 16 * 1000000; // 16 milliseconds
    loop {
        process_websocket_events(&mut game_state, &game_messages);

        process_game_update(&mut game_state);

        send_state_updates(&game_state);

        // Sleep if needed to the next update
        let time_till_next = (((iter + 1) * iter_length) as i64) -
                             ((time::precise_time_ns() - start_time) as i64);
        iter += 1;
        if time_till_next > 0 {
            std::thread::sleep(Duration::new(0, time_till_next as u32));
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut host = String::from("127.0.0.1");
    let mut port = 8080;

    if args.len() >= 2 {
        host = args[1].clone();
    }

    if args.len() >= 3 {
        port = args[2].parse::<u16>().unwrap();
    }

    // Create the channel which will allow the game loop to recieve messages.
    let (tx, rx) = channel::<WebSocketEvent>();

    let _ = thread::spawn(move || {
        game_loop(rx);
    });

    listen(&host, port, tx);
}
