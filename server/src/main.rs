#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    unsafe_code,
)]


extern crate serde;
extern crate serde_json;

extern crate time;
extern crate websocket;

use serde_json::builder::ObjectBuilder;

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::vec::Vec;

use websocket::message::Type;
use websocket::{Server, Message, Sender, Receiver};
use websocket::server::Connection;
use websocket::stream::WebSocketStream;

/*
  A game entity is simply something with an x and y position.
  Right now there are only player entities.
  The player id is right now the ip address/port.
  NOTE: There is a risk of overlap here if people leave. This needs fixing.
  */
#[derive(Debug)]
struct Entity {
    id: String,
    x: f64,
    y: f64,
}

/*
  A GameMessage is any message which might be sent to the main game loop.
  Right now, we have clients connecting, disconnecting, and sending messages.
  This is the place where we would add additional stuff like say, unix signals.
*/
#[derive(Debug)]
enum GameMessage {
    ClientCreated(Client),
    ClientClosed(String),
    ClientMessage(String, String),
}

/*
  This represends a single websocket connected to the game.
  The id is simply the ip address and port.
  'sender' is a channel which allows you to send messages to the corresponding websocket.
  Send a None to close the websocket. (Some(data) for a normal message).
  */
struct Client {
    id: String, 
    sender: std::sync::mpsc::Sender<Option<String>>,
    pressed_keys: HashSet<String>,
}

impl Client {
    /* 
      Create a new client from a given id and sender channel.
    */
    fn new(id: String, sender: std::sync::mpsc::Sender<Option<String>>) -> Client {
        Client {
            id: id,
            sender: sender,
            pressed_keys: HashSet::new(),
        }
    }

    /*
      Send a message to the websocket.
    */
    fn send(&self, message: String) {
        self.sender.send(Some(message)).unwrap();
    }

    /*
      Close the websocket.
    */
    fn close(&self) {
        self.sender.send(None).unwrap();
    }

    /*
      Manipulate the pressed keys with keydown and keyup events.
    */
    fn keydown(&mut self, key: String) {
        self.pressed_keys.insert(key);
    }

    fn keyup(&mut self, key: String) {
        self.pressed_keys.remove(&key);
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Client {}", self.id)
    }
}

/*
  The GameState contains the whole state of the game.
  It consists of both entities, and all the clients which are currently connected.
*/
#[derive(Debug)]
struct GameState {
    entities: HashMap<String, Entity>,
    clients: HashMap<String, Client>,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            entities: HashMap::new(),
            clients: HashMap::new(),
        }
    }
}

/*
  Serialize the entire game state into one json object.
*/
fn serialize_state(game_state: &GameState) -> serde_json::value::Value {
    ObjectBuilder::new()
        .insert_array("entities", |builder| {
            game_state.entities.iter().fold(builder, |builder, (_, entity)| 
                builder.push_object(|builder| {
                    // Simply add the id, x, and y for each entity.
                    // TODO: This should be a map, not an array.
                    builder.insert("id", entity.id.clone()).insert("x", entity.x).insert("y", entity.y)
                })
            )
        })
        .unwrap()
}

/*
  Process a simple string message from the client.
  TODO: All this manual json parsing is error prone, find a better way to do it.
*/
fn process_client_message(game_state: &mut GameState, client_id: String, message: String) {
    let parsed_json: serde_json::value::Value = serde_json::de::from_str(&message).unwrap();

    let event_type = parsed_json.find("type").unwrap().as_string().unwrap();

    match event_type {
        "keydown" => {
            let key = parsed_json.find("key").unwrap().as_string().unwrap().to_string();
            game_state.clients.get_mut(&client_id).unwrap().keydown(key);
        },
        "keyup" => {
            let key = parsed_json.find("key").unwrap().as_string().unwrap().to_string();
            game_state.clients.get_mut(&client_id).unwrap().keyup(key);
        },
        _ => panic!("Unexpected event type {}.", event_type),
    }
}

/*
  Process a GameMessage (which is any message which effects the game)
*/
fn process_game_message(game_state: &mut GameState, message: GameMessage) {
    match message {
        GameMessage::ClientCreated(new_client) => {
            game_state.entities.insert(new_client.id.clone(), Entity { id: new_client.id.clone(), x: 0.0, y: 0.0 });
            game_state.clients.insert(new_client.id.clone(), new_client);
        },
        GameMessage::ClientClosed(client_id) => {
            game_state.clients.remove(&client_id);
        },
        GameMessage::ClientMessage(client_id, message) => {
            process_client_message(game_state, client_id, message);
        },
    }
}

/*
  Tries to process every available game message without blocking.
*/
fn process_game_loop_messages(game_state: &mut GameState, game_messages: &std::sync::mpsc::Receiver<GameMessage>) {
    loop {
        match game_messages.try_recv() {
            Ok(a) => process_game_message(game_state, a),
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
    for (_, client) in &game_state.clients {
        if client.pressed_keys.contains("ArrowUp") {
            game_state.entities.get_mut(&client.id).unwrap().y -= 1.0;
        }
        if client.pressed_keys.contains("ArrowDown") {
            game_state.entities.get_mut(&client.id).unwrap().y += 1.0;
        }
        if client.pressed_keys.contains("ArrowLeft") {
            game_state.entities.get_mut(&client.id).unwrap().x -= 1.0;
        } 
        if client.pressed_keys.contains("ArrowRight") {
            game_state.entities.get_mut(&client.id).unwrap().x += 1.0;
        }
    }
}

/*
  Send the current, entire state to each client.
*/
fn send_state_updates(game_state: &GameState) {
    let value = serialize_state(game_state);

    let result = serde_json::ser::to_string(&value).unwrap();

    for (_, client) in &game_state.clients {
        client.send(result.clone());
    }
}

/*
  Runs the main game loop.

  The general idea for the game loop is to update the game state every 20 seconds, processing messages along the way.
*/

fn game_loop(game_messages: std::sync::mpsc::Receiver<GameMessage>) {
    let mut game_state = GameState::new();

    let start_time = time::precise_time_ns();
    let mut iter: u64 = 0;
    let iter_length: u64 = 20 * 1000000; // 20 milliseconds
    loop {
        process_game_loop_messages(&mut game_state, &game_messages);

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

/*
  Handle a given connection.
  The basic idea is what we create two infinite loops:
    One which forever reads from the game loop via a channel and sends stuff to the websocket when requested.
    And one which forever reads from a websocket and sends the stuff to the game loop via a channel.
*/
fn handle_connection(connection: std::io::Result<Connection<WebSocketStream, WebSocketStream>>, game_messages_sender: std::sync::mpsc::Sender<GameMessage>) {
    let request = connection.unwrap().read_request().unwrap(); // Get the request

    request.validate().unwrap(); // Validate the request

    let response = request.accept(); // Form a response

    let mut client = response.send().unwrap(); // Send the response

    let ip = client.get_mut_sender()
        .get_mut()
        .peer_addr()
        .unwrap();

    let id = ip.to_string();

    println!("Connection from {}", id);

    let (mut sender, mut receiver) = client.split();

    // Create the channel which will allow the game loop to send messages to websockets.
    let (tx, rx) = channel::<Option<String>>();

    game_messages_sender.send(GameMessage::ClientCreated(Client::new(id.clone(), tx))).unwrap();

    // Create the thread for sending websocket messages.
    thread::spawn(move || {
        for message in rx {
            match message {
                Some(text) => {sender.send_message(&Message::text(text)).unwrap();},
                None => {
                    sender.send_message(&Message::close()).unwrap();
                    return;
                },
            }
        }
    });

    // Handle all incoming messages by forwarding them to the game loop.
    for message in receiver.incoming_messages() {
        let message: Message = message.unwrap();

        match message.opcode {
            Type::Close => {
                println!("Client {} disconnected", ip);
                game_messages_sender.send(GameMessage::ClientClosed(id.clone())).unwrap();
                return;
            },
            Type::Text => {
                let text = std::str::from_utf8(&message.payload).unwrap().to_string();
                game_messages_sender.send(GameMessage::ClientMessage(id.clone(), text)).unwrap();
            }
            _ => {
                panic!("Unknown message type {:?}", message);
            }
        }
    }
}

/*
  The main listening loop for the server.
*/
fn listen(host: &str, port: u16, game_messages_sender: std::sync::mpsc::Sender<GameMessage>) {
    println!("Listening on {}:{}", host, port);
    let server = Server::bind((host, port)).unwrap();

    for connection in server {
        let temp = game_messages_sender.clone();
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            handle_connection(connection, temp);
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
    let (tx, rx) = channel::<GameMessage>();

    thread::spawn(move || {
        game_loop(rx);
    });

    listen(&host, port, tx);
}
