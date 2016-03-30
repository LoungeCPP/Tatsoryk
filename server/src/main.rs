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

use websocket::message::Type;
use websocket::{Server, Message, Sender, Receiver};
use websocket::server::Connection;
use websocket::stream::WebSocketStream;

use std::iter::FromIterator;
use std::str::FromStr;

/*
  A WebSocketEvent is any websocket message which might be sent to the main game loop.
  Right now, we have clients connecting, disconnecting, and sending messages.
  This is the place where we would add additional stuff like say, unix signals.
*/
#[derive(Debug)]
enum WebSocketEvent {
    ClientCreated {client: Client},
    ClientClosed {client_id: u32},
    ClientMessage {client_id: u32, message: message::Message},
}

/*
  This represends a single websocket connected to the game.
  The id is simply the ip address and port.
  'sender' is a channel which allows you to send messages to the corresponding websocket.
  Send a None to close the websocket. (Some(data) for a normal message).
  */
struct Client {
    id: u32, 
    sender: std::sync::mpsc::Sender<Option<String>>,
}

impl Client {
    /* 
      Create a new client from a given id and sender channel.
    */
    fn new(id: u32, sender: std::sync::mpsc::Sender<Option<String>>) -> Client {
        let result = Client {
            id: id,
            sender: sender,
        };
        result
    }

    /*
      Send a message to the websocket.
    */
    fn send(&self, message: String) -> Result<(), std::sync::mpsc::SendError<Option<String>>> {
        self.sender.send(Some(message))
    }

    // /*
    //   Close the websocket.
    // */
    // fn close(&self) -> Result<(), std::sync::mpsc::SendError<Option<String>>> {
    //     self.sender.send(None)
    // }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Client {}", self.id)
    }
}

/*
  The GameState contains the whole state of the game.
  It consists of both players, and all the clients which are currently connected.
*/
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

/*
  Serialize the entire game state into one json string.
*/
fn serialize_state(game_state: &GameState) -> String {
    let players: Vec<message::Player> = Vec::from_iter(game_state.players.values().map(|a| a.clone()));
    let state = message::Message::WorldState {
        player_count: players.len() as u32,
        alive_players: players,
        alive_bullets: Vec::new(),
    };
    state.to_string()
}

/*
  Process a simple string message from the client.
*/
fn process_client_message(game_state: &mut GameState, client_id: u32, message: message::Message) {
    match message {
        message::Message::StartMoving {move_x, move_y} => {
            let player = game_state.players.get_mut(&client_id).unwrap();
            player.move_x = Some(move_x);
            player.move_y = Some(move_y);
        },
        _ => panic!("Unprocessed message! {}", message.to_string())
    }
}

/*
  Process a web socket event.
*/
fn process_websocket_event(game_state: &mut GameState, message: WebSocketEvent) {
    match message {
        WebSocketEvent::ClientCreated { client } => {
            let _ = game_state.players.insert(client.id.clone(), 
                message::Player::not_moving(client.id.clone(), 0.0, 0.0));
            let _ = game_state.clients.insert(client.id.clone(), client);
        },
        WebSocketEvent::ClientClosed {client_id } => {
            let _ = game_state.clients.remove(&client_id);
        },
        WebSocketEvent::ClientMessage { client_id, message } => {
            process_client_message(game_state, client_id, message);
        },
    }
}

/*
  Tries to process every available websocket event without blocking.
*/
fn process_websocket_events(game_state: &mut GameState, game_messages: &std::sync::mpsc::Receiver<WebSocketEvent>) {
    loop {
        match game_messages.try_recv() {
            Ok(a) => process_websocket_event(game_state, a),
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => return,
                std::sync::mpsc::TryRecvError::Disconnected => panic!("Now I am disconnected?"),
            }
        }
    }
}

/*
  Updates the game state in one tick.
*/
fn process_game_update(game_state: &mut GameState) {
    for (_, player) in &mut game_state.players {
        player.x += player.move_x.unwrap_or(0.0);
        player.y += player.move_y.unwrap_or(0.0);
    }
}

/*
  Send the current, entire state to each client.
*/
fn send_state_updates(game_state: &GameState) {
    let value = serialize_state(game_state);

    for (_, client) in &game_state.clients {
        // Always ignore if the send fails.
        // We will eventually get a disconnect WebSocketMessage where we will cleanly do the disconnect.
        let _ = client.send(value.clone());
    }
}

/*
  Runs the main game loop.

  The general idea for the game loop is to update the game state every 16 milliseconds (60 FPS), processing messages along the way.
*/

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
        let time_till_next = (((iter + 1) * iter_length) as i64) - ((time::precise_time_ns() - start_time) as i64);
        iter += 1;
        if time_till_next > 0 {
            std::thread::sleep(Duration::new(0, time_till_next as u32));
        }
    }
}

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

/*
  Constantly send messages over the websocket.
*/
fn websocket_send_loop<S: Sender>(rx: std::sync::mpsc::Receiver<Option<String>>, mut sender: S) -> Result<(), AllErrors> {
    for message in rx {
        match message {
            Some(text) => {try!(sender.send_message(&Message::text(text)));},
            None => {
                try!(sender.send_message(&Message::close()));
                return Ok(());
            },
        }
    }

    return Ok(());
}


/*
  Handle a given connection.
  The basic idea is what we create two infinite loops:
    One which forever reads from the game loop via a channel and sends stuff to the websocket when requested.
    And one which forever reads from a websocket and sends the stuff to the game loop via a channel.
*/
fn handle_connection(
    id: u32, 
    connection: std::io::Result<Connection<WebSocketStream, WebSocketStream>>, 
    game_messages_sender: std::sync::mpsc::Sender<WebSocketEvent>
) -> Result<(), AllErrors> {
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
    game_messages_sender.send(WebSocketEvent::ClientCreated{client :Client::new(id, tx)}).unwrap();

    // Create the thread for sending websocket messages.
    let _ = thread::spawn(move || {
        match websocket_send_loop(rx, sender) {
            Ok(()) => {}, // ignore
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
                game_messages_sender.send(WebSocketEvent::ClientClosed {client_id: id.clone()}).unwrap();
                return Ok(());
            },
            Type::Text => {
                let text = std::str::from_utf8(&message.payload).unwrap().to_string();

                // Should never fail
                game_messages_sender.send(
                    WebSocketEvent::ClientMessage {client_id: id.clone(), message: message::Message::from_str(&text).unwrap()}
                ).unwrap();
            }
            _ => {
                panic!("Unknown message type {:?}", message);
            }
        }
    }

    return Ok(());
}

/*
  The main listening loop for the server.
*/
fn listen(host: &str, port: u16, game_messages_sender: std::sync::mpsc::Sender<WebSocketEvent>) {
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
                Ok(_) => {},
            };
        });
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
