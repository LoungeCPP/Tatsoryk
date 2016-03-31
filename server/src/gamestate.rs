//! The state of the game itself.

use message;

use std::collections::HashMap;
use std::sync::mpsc;

use std::vec::Vec;

use std::iter::FromIterator;

use events::Client;
use events::WebSocketEvent;

const BULLET_RADIUS: f32 = 5.0;
const PLAYER_RADIUS: f32 = 10.0;

/// The GameState contains the whole state of the game.
///
/// It consists of both players, and all the clients which are currently connected.
#[derive(Debug)]
pub struct GameState {
    players: HashMap<u32, message::Player>,
    bullets: HashMap<u32, message::Bullet>,
    clients: HashMap<u32, Client>,
    next_bullet_id: u32,
}

impl GameState {
    /// Create a new game state.
    pub fn new() -> GameState {
        GameState {
            players: HashMap::new(),
            bullets: HashMap::new(),
            clients: HashMap::new(),
            next_bullet_id: 0,
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
        // Do a normal position update
        for (_, player) in &mut self.players {
            player.x += player.move_x.unwrap_or(0.0);
            player.y += player.move_y.unwrap_or(0.0);
        }

        let mut destroyed_bullets: Vec<u32> = Vec::new();
        let mut destroyed_players: Vec<u32> = Vec::new();

        for (_, bullet) in &mut self.bullets {
            bullet.x += bullet.move_x.unwrap_or(0.0);
            bullet.y += bullet.move_y.unwrap_or(0.0);

            if bullet.x < 0.0 || bullet.x > 500.0 || bullet.y < 0.0 || bullet.y > 500.0 {
                destroyed_bullets.push(bullet.id);
            }
        }

        let mut destroyed: Vec<(u32, u32)> = Vec::new();

        // Check for collisions
        for (_, bullet) in &mut self.bullets {
            for (_, player) in &mut self.players {
                let dx = bullet.x - player.x;
                let dy = bullet.y - player.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < BULLET_RADIUS + PLAYER_RADIUS {
                    destroyed_bullets.push(bullet.id);
                    destroyed_players.push(player.id);
                }
            }
        }

        // Process destroy requests
        for bullet_id in destroyed_bullets {
            let _ = self.bullets.remove(&bullet_id);
        }

        for player_id in destroyed_players {
            let dead_player = self.players.get_mut(&player_id).unwrap();
            // Respawn the player at 50, 50
            dead_player.x = 50.0;
            dead_player.y = 50.0;
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
                let welcome_message = message::Message::Welcome {
                    id: client.id.clone(),
                    speed: 0.0,
                    size: PLAYER_RADIUS,
                    bullet_speed: 0.0,
                    bullet_size: BULLET_RADIUS,
                };

                let _ = client.send(welcome_message.to_string());
                let _ = client.send(self.serialize());

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
        let players_clone = Vec::from_iter(self.players
                                               .values()
                                               .map(Clone::clone));
        let bullets_clone = Vec::from_iter(self.bullets
                                               .values()
                                               .map(Clone::clone));
        let state = message::Message::WorldState {
            player_count: players_clone.len() as u32,
            alive_players: players_clone,
            alive_bullets: bullets_clone,
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
            message::Message::Fire { move_x, move_y } => {
                let player = self.players.get(&client_id).unwrap();

                // Have to move the bullet out of the way of the player to avoid an instant collision.
                let start_x = player.x + move_x * (BULLET_RADIUS + PLAYER_RADIUS + 1.0);
                let start_y = player.y + move_y * (BULLET_RADIUS + PLAYER_RADIUS + 1.0);

                let _ = self.bullets.insert(self.next_bullet_id,
                                            message::Bullet::moving(self.next_bullet_id,
                                                                    start_x,
                                                                    start_y,
                                                                    move_x,
                                                                    move_y));

                self.next_bullet_id += 1;

            }
            _ => panic!("Unprocessed message! {}", message.to_string()),
        }
    }
}
