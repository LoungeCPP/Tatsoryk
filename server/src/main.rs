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
extern crate ctrlc;
extern crate serde;
extern crate serde_json;
extern crate websocket;

mod options;
pub mod math;
pub mod message;
pub mod server;

use websocket::Client;
use websocket::client::request::Url;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;

use server::{listen, start_game_loop};
pub use options::Options;

fn main() {
    let opts = Options::parse();

    let cont = Arc::new(RwLock::new(true));

    {
        let host = opts.host.clone();
        let port = opts.port;
        let cont = cont.clone();
        ctrlc::set_handler(move || {
            println!("Ctrl+C received, terminating...");
            *cont.write().unwrap() = false;
            let _ = Client::connect(Url::parse(&format!("ws://{}:{}", host, port)[..]).unwrap());
        });
    }

    // Create the channel which will allow the game loop to recieve messages.
    let (tx, rx) = channel();

    let game_loop_handle = start_game_loop(rx, &cont);
    listen(&opts.host, opts.port, tx, &cont);
    if let Err(error) = game_loop_handle.join() {
        println!("Game loop thread failed: {:?}", error);
    }
}
