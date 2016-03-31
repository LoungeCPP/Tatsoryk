//! The server logic for the game.
//!
//! In order for the multiplayer to work, the server program listens for websocket connections.
//! This file implements that logic.

use std::sync::mpsc::channel;
use std::thread;

use websocket;
use std;
use message;

use websocket::message::Type;
use websocket::{Server, Message, Receiver};
use websocket::server::Connection;
use websocket::stream::WebSocketStream;
use std::sync::mpsc;

use std::str::FromStr;

use events::Client;
use events::WebSocketEvent;

#[derive(Debug)]
enum AllErrors {
    WebSocketError(websocket::result::WebSocketError),
    IoError(std::io::Error),
}

impl From<std::io::Error> for AllErrors {
    fn from(error: std::io::Error) -> AllErrors {
        AllErrors::IoError(error)
    }
}

impl From<websocket::result::WebSocketError> for AllErrors {
    fn from(error: websocket::result::WebSocketError) -> AllErrors {
        AllErrors::WebSocketError(error)
    }
}

/// Constantly send messages over the websocket.
fn websocket_send_loop<S: websocket::Sender>(rx: mpsc::Receiver<Option<String>>,
                                  mut sender: S)
                                  -> Result<(), AllErrors> {
    for message in rx {
        match message {
            Some(text) => {
                try!(sender.send_message(&Message::text(text)));
            }
            None => {
                try!(sender.send_message(&Message::close()));
                return Ok(());
            }
        }
    }

    return Ok(());
}


/// Handle a given connection.
///
/// The basic idea is what we create two infinite loops:
/// One which forever reads from the game loop via a channel and sends stuff to the websocket when requested.
/// And one which forever reads from a websocket and sends the stuff to the game loop via a channel.
fn handle_connection(id: u32,
                     connection: std::io::Result<Connection<WebSocketStream, WebSocketStream>>,
                     game_messages_sender: mpsc::Sender<WebSocketEvent>)
                     -> Result<(), AllErrors> {
    let request = try!(connection.unwrap().read_request()); // Get the request

    try!(request.validate()); // Validate the request

    let response = request.accept(); // Form a response

    let mut client = try!(response.send()); // Send the response

    let ip = client.get_mut_sender()
                   .get_mut()
                   .peer_addr()
                   .unwrap();

    println!("Connection from {}", id);

    let (sender, mut receiver) = client.split();

    // Create the channel which will allow the game loop to send messages to websockets.
    let (tx, rx) = channel::<Option<String>>();

    // Should never fail
    game_messages_sender.send(WebSocketEvent::ClientCreated { client: Client::new(id, tx) })
                        .unwrap();

    // Create the thread for sending websocket messages.
    let _ = thread::spawn(move || {
        match websocket_send_loop(rx, sender) {
            Ok(()) => {} // ignore
            Err(e) => panic!("Send loop had an error for client {} , {:?}", id, e),
        }
    });

    // Handle all incoming messages by forwarding them to the game loop.
    for message in receiver.incoming_messages() {
        let message: Message = try!(message);

        match message.opcode {
            Type::Close => {
                println!("Client {} disconnected", ip);

                // Should never fail
                game_messages_sender.send(WebSocketEvent::ClientClosed { client_id: id.clone() })
                                    .unwrap();
                return Ok(());
            }
            Type::Text => {
                let text = std::str::from_utf8(&message.payload).unwrap().to_string();

                // Should never fail
                game_messages_sender.send(WebSocketEvent::ClientMessage {
                                        client_id: id.clone(),
                                        message: message::Message::from_str(&text).unwrap(),
                                    })
                                    .unwrap();
            }
            _ => {
                panic!("Unknown message type {:?}", message);
            }
        }
    }

    return Ok(());
}

/// The main listening loop for the server.
pub fn listen(host: &str,
              port: u16,
              game_messages_sender: mpsc::Sender<WebSocketEvent>) {
    println!("Listening on {}:{}", host, port);
    let server = Server::bind((host, port)).unwrap();

    let mut next_client_id = 0;

    for connection in server {
        let temp = game_messages_sender.clone();
        let id = next_client_id;
        next_client_id += 1;
        // Spawn a new thread for each connection.
        let _ = thread::spawn(move || {
            match handle_connection(id, connection, temp) {
                Err(e) => panic!("Connection {} quit with error {:?}", id, e),
                Ok(_) => {}
            };
        });
    }

}
