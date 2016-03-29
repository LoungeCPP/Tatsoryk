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

extern crate serde_json;
extern crate serde;

pub mod message;

use std::env;
use std::vec::Vec;

fn listen(host: &str, port: i32) {
    println!("Listening on {}:{}", host, port);
    // TODO actually listen
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut host = String::from("127.0.0.1");
    let mut port = 8080;

    if args.len() >= 2 {
        host = args[1].clone();
    }

    if args.len() >= 3 {
        port = args[2].parse::<i32>().unwrap();
    }

    listen(&host, port);
}
