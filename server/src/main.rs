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
extern crate clap;

mod options;
pub mod message;

pub use options::Options;

fn listen(host: &str, port: u16) {
    println!("Listening on {}:{}", host, port);
    // TODO actually listen
}

fn main() {
    let opts = Options::parse();
    listen(&opts.host, opts.port);
}
