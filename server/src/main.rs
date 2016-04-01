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

extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate websocket;

mod options;
pub mod message;
pub mod events;
pub mod server;
pub mod gamestate;

use std::sync::mpsc::{self, channel};
use std::thread;
use std::time::Duration;

use events::WebSocketEvent;

use server::listen;
use gamestate::GameState;

use options::Options;

/// Runs the main game loop.
///
/// The general idea for the game loop is to update the game state every 16 milliseconds (60 FPS), processing messages along the way.
fn game_loop(game_messages: mpsc::Receiver<WebSocketEvent>, player_size: f32, bullet_size: f32) {
    static ITER_LENGTH: u64 = 16 * 1000000; // 16 milliseconds

    let mut game_state = GameState::new(player_size, bullet_size);

    let start_time = time::precise_time_ns();
    let mut iter: u64 = 1;
    loop {
        game_state.process_websocket_events(&game_messages);
        game_state.process_game_update();
        game_state.send_state_updates();

        // Sleep if needed to the next update
        let time_till_next = ((iter * ITER_LENGTH) as i64) -
                             ((time::precise_time_ns() - start_time) as i64);
        iter += 1;
        if time_till_next > 0 {
            thread::sleep(Duration::new(0, time_till_next as u32));
        }
    }
}

fn main() {
    let opts = Options::parse();

    // Create the channel which will allow the game loop to recieve messages.
    let (tx, rx) = channel();

    {
        let player_size = opts.player_size;
        let bullet_size = opts.bullet_size;
        let _ = thread::spawn(move || {
            game_loop(rx, player_size, bullet_size);
        });
    }

    listen(&opts.host, opts.port, tx);
}
