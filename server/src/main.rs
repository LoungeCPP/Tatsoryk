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
extern crate rand;
extern crate time;
extern crate serde;
extern crate serde_json;
extern crate websocket;

mod options;
pub mod math;
pub mod message;
pub mod server;

use std::sync::mpsc::channel;

use server::{listen, start_game_loop};
pub use options::Options;

fn main() {
    let opts = Options::parse();

    // Create the channel which will allow the game loop to recieve messages.
    let (tx, rx) = channel();

    start_game_loop(rx);
    listen(&opts.host, opts.port, tx);
}
