//! The state of the game itself.

use message;

use std::collections::HashMap;
use std::sync::mpsc;

use std::vec::Vec;

use std::iter::FromIterator;

use events::Client;
use events::WebSocketEvent;

/// The GameState contains the whole state of the game.
///
/// It consists of both players, and all the clients which are currently connected.
#[derive(Debug)]
pub struct GameState {
    players: HashMap<u32, message::Player>,
    clients: HashMap<u32, Client>,
}

impl GameState {
    /// Create a new game state.
    pub fn new() -> GameState {
        GameState {
            players: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    /// Tries to process every available websocket event without blocking.
    pub fn process_websocket_events(&mut self, game_messages: &mpsc::Receiver<WebSocketEvent>) {
        loop {
            match game_messages.try_recv() {
                Ok(message) => self.process_websocket_event(message),
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => panic!("Now I am disconnected?"),
            }
        }
    }

    /// Updates the game state in one tick.
    pub fn process_game_update(&mut self) {
        for (_, player) in &mut self.players {
            player.x += player.move_x.unwrap_or(0.0);
            player.y += player.move_y.unwrap_or(0.0);
        }
    }

    /// Send the current state to each client.
    pub fn send_state_updates(&self) {
        let value = self.serialize();

        for (_, client) in &self.clients {
            // Always ignore if the send fails.
            // We will eventually get a disconnect WebSocketMessage where we will cleanly do the disconnect.
            let _ = client.send(value.clone());
        }
    }

    /// Process a web socket event.
    fn process_websocket_event(&mut self, message: WebSocketEvent) {
        match message {
            WebSocketEvent::ClientCreated { client } => {
                let _ = self.players.insert(client.id.clone(),
                                            message::Player::not_moving(client.id.clone(),
                                                                        0.0,
                                                                        0.0));
                let _ = self.clients.insert(client.id.clone(), client);
            }
            WebSocketEvent::ClientClosed { client_id } => {
                let _ = self.clients.remove(&client_id);
            }
            WebSocketEvent::ClientMessage { client_id, message } => {
                self.process_client_message(client_id, message);
            }
        }
    }

    /// Serialize the entire game state into one json string.
    fn serialize(&self) -> String {
        let players = Vec::from_iter(self.players
                                         .values()
                                         .map(Clone::clone));
        let state = message::Message::WorldState {
            player_count: players.len() as u32,
            alive_players: players,
            alive_bullets: Vec::new(),
        };
        state.to_string()
    }

    /// Process a simple string message from the client.
    fn process_client_message(&mut self, client_id: u32, message: message::Message) {
        match message {
            message::Message::StartMoving { move_x, move_y } => {
                let player = self.players.get_mut(&client_id).unwrap();
                player.move_x = Some(move_x);
                player.move_y = Some(move_y);
            }
            _ => panic!("Unprocessed message! {}", message.to_string()),
        }
    }
}
