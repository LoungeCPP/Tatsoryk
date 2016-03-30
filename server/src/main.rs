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
pub mod gamestate;

extern crate time;
extern crate websocket;
extern crate serde;
extern crate serde_json;

use std::env;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::vec::Vec;

use events::WebSocketEvent;

use server::listen;
use gamestate::GameState;

/// Runs the main game loop.
///
/// The general idea for the game loop is to update the game state every 16 milliseconds (60 FPS), processing messages along the way.
fn game_loop(game_messages: std::sync::mpsc::Receiver<WebSocketEvent>) {
    let mut game_state = GameState::new();

    let start_time = time::precise_time_ns();
    let mut iter: u64 = 0;
    let iter_length: u64 = 16 * 1000000; // 16 milliseconds
    loop {
        game_state.process_websocket_events(&game_messages);

        game_state.process_game_update();

        game_state.send_state_updates();

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
