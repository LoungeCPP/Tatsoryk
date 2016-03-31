//! The server logic for the game.
//!
//! In order for the multiplayer to work, the server program listens for websocket connections.
//! This file implements that logic.

use std::sync::mpsc::channel;
use std::thread;

use websocket;
use message;
use std::io;

use websocket::message::Type;
use websocket::{Server, Message, Receiver};
use websocket::server::Connection;
use websocket::stream::WebSocketStream;
use std::sync::mpsc;

use std::str::{self, FromStr};

use events::Client;
use events::WebSocketEvent;

/// The main listening loop for the server.
pub fn listen(host: &str, port: u16, game_messages_sender: mpsc::Sender<WebSocketEvent>) {
    println!("Listening on {}:{}", host, port);
    let server = Server::bind((host, port)).unwrap();

    let mut next_client_id = 0;

    for connection in server {
        let temp = game_messages_sender.clone();
        let id = next_client_id;
        next_client_id += 1;
        // Spawn a new thread for each connection.
        let _ = thread::spawn(move || {
            if let Err(e) = handle_connection(id, connection, temp) {
                panic!("Connection {} quit with error {:?}", id, e)
            }
        });
    }

}

#[derive(Debug)]
enum ServerError {
    WebSocketError(websocket::result::WebSocketError),
    IoError(io::Error),
}

impl From<io::Error> for ServerError {
    fn from(error: io::Error) -> ServerError {
        ServerError::IoError(error)
    }
}

impl From<websocket::result::WebSocketError> for ServerError {
    fn from(error: websocket::result::WebSocketError) -> ServerError {
        ServerError::WebSocketError(error)
    }
}

/// Handle a given connection.
///
/// The basic idea is what we create two infinite loops:
/// One which forever reads from the game loop via a channel and sends stuff to the websocket when requested.
/// And one which forever reads from a websocket and sends the stuff to the game loop via a channel.
fn handle_connection(id: u32,
                     connection: io::Result<Connection<WebSocketStream, WebSocketStream>>,
                     game_messages_sender: mpsc::Sender<WebSocketEvent>)
                     -> Result<(), ServerError> {
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
    let (tx, rx) = channel();

    // Should never fail
    game_messages_sender.send(WebSocketEvent::ClientCreated { client: Client::new(id, tx) })
                        .unwrap();

    // Create the thread for sending websocket messages.
    let _ = thread::spawn(move || {
        if let Err(e) = websocket_send_loop(rx, sender) {
            panic!("Send loop had an error for client {} , {:?}", id, e)
        }
    });

    // Handle all incoming messages by forwarding them to the game loop.
    for message in receiver.incoming_messages() {
        let message: Message = try!(message);

        match message.opcode {
            Type::Close => {
                println!("Client {} disconnected", ip);

                // Should never fail
                game_messages_sender.send(WebSocketEvent::ClientClosed { client_id: id })
                                    .unwrap();
                return Ok(());
            }
            Type::Text => {
                let text = str::from_utf8(&message.payload).unwrap();

                // Should never fail
                game_messages_sender.send(WebSocketEvent::ClientMessage {
                                        client_id: id,
                                        message: message::Message::from_str(text).unwrap(),
                                    })
                                    .unwrap();
            }
            _ => {
                panic!("Unknown message type {:?}", message);
            }
        }
    }

    Ok(())
}

/// Constantly send messages over the websocket.
fn websocket_send_loop<S: websocket::Sender>(rx: mpsc::Receiver<Option<String>>,
                                             mut sender: S)
                                             -> Result<(), ServerError> {
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

    Ok(())
}
