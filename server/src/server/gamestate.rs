//! The state of the game itself.

use message;

use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::mpsc;

use std::vec::Vec;

use std::iter::FromIterator;

use math::distance_between;
use rand::{thread_rng, Rng};

use self::super::Client;
use self::super::WebSocketEvent;

static BULLET_RADIUS: f32 = 5.0;
static PLAYER_RADIUS: f32 = 10.0;
static MAP_WIDTH: f32 = 500.0;
static MAP_HEIGHT: f32 = 500.0;

/// The `GameState` contains the whole state of the game.
///
/// It consists of both players, and all the clients which are currently connected.
#[derive(Debug)]
pub struct GameState {
    players: HashMap<u32, RefCell<message::Player>>,
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
        for (_, player_cell) in &self.players {
            let mut player = player_cell.borrow_mut();
            let temp_x = player.x + player.move_x.unwrap_or(0.0);
            let temp_y = player.y + player.move_y.unwrap_or(0.0);

            // Check borders
            if temp_x < (0.0 + PLAYER_RADIUS) || temp_x > (MAP_WIDTH - PLAYER_RADIUS) ||
               temp_y < (0.0 + PLAYER_RADIUS) ||
               temp_y > (MAP_HEIGHT - PLAYER_RADIUS) {
                continue;
            }

            let mut colliding = false;

            // Check collisions
            for (player2_id, player2_cell) in &self.players {
                if *player2_id == player.id {
                    continue;
                }
                let player2 = player2_cell.borrow();
                if distance_between(temp_x, temp_y, player2.x, player2.y) < 2.0 * PLAYER_RADIUS {
                    colliding = true;
                }
            }

            if colliding {
                continue;
            }

            // Only update now that we have verified that it is valid
            player.x = temp_x;
            player.y = temp_y;

        }

        let mut destroyed_bullets = Vec::new();
        let mut destroyed_players = Vec::new();

        for (_, bullet) in &mut self.bullets {
            bullet.x += bullet.move_x.unwrap_or(0.0);
            bullet.y += bullet.move_y.unwrap_or(0.0);

            if bullet.x < 0.0 || bullet.x > MAP_WIDTH || bullet.y < 0.0 || bullet.y > MAP_HEIGHT {
                destroyed_bullets.push(bullet.id);
            }
        }

        // Check for bullet collisions
        for (_, bullet) in &self.bullets {
            for (_, player_cell) in &self.players {
                if distance_between(bullet.x,
                                    bullet.y,
                                    player_cell.borrow().x,
                                    player_cell.borrow().y) <
                   BULLET_RADIUS + PLAYER_RADIUS {
                    destroyed_bullets.push(bullet.id);
                    destroyed_players.push(player_cell.borrow().id);
                }
            }
        }

        // Process destroy requests
        for bullet_id in destroyed_bullets {
            let _ = self.bullets.remove(&bullet_id);
        }

        let mut rng = thread_rng();
        for player_id in destroyed_players {
            let (new_x, new_y) = self.random_free_spot(&mut rng);
            let mut dead_player = self.players.get_mut(&player_id).unwrap().borrow_mut();
            dead_player.x = new_x;
            dead_player.y = new_y;
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
                    id: client.id,
                    speed: 0.0,
                    size: PLAYER_RADIUS,
                    bullet_speed: 0.0,
                    bullet_size: BULLET_RADIUS,
                };

                let _ = client.send(welcome_message.to_string());
                let _ = client.send(self.serialize());

                let (x, y) = self.random_free_spot(&mut thread_rng());
                let _ = self.players
                            .insert(client.id,
                                    RefCell::new(message::Player::not_moving(client.id, x, y)));

                let _ = self.clients.insert(client.id, client);
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
                                         .map(|a| a.borrow().clone()));
        let bullets = Vec::from_iter(self.bullets
                                         .values()
                                         .map(Clone::clone));
        let state = message::Message::WorldState {
            player_count: players.len() as u32,
            alive_players: players,
            alive_bullets: bullets,
        };
        state.to_string()
    }

    /// Process a simple string message from the client.
    fn process_client_message(&mut self, client_id: u32, message: message::Message) {
        match message {
            message::Message::StartMoving { move_x, move_y } => {
                let mut player = self.players.get_mut(&client_id).unwrap().borrow_mut();
                player.move_x = Some(move_x);
                player.move_y = Some(move_y);
            }
            message::Message::StopMoving => {
                let mut player = self.players.get_mut(&client_id).unwrap().borrow_mut();
                player.move_x = None;
                player.move_y = None;
            }
            message::Message::Fire { move_x, move_y } => {
                let player = self.players.get(&client_id).unwrap().borrow();

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

    fn random_free_spot<R: Rng>(&self, rng: &mut R) -> (f32, f32) {
        static MAX_ITERATIONS: u32 = 100;

        for _ in 1..MAX_ITERATIONS {
            let x: f32 = rng.gen_range(PLAYER_RADIUS + 1.0, MAP_WIDTH - PLAYER_RADIUS - 1.0);
            let y: f32 = rng.gen_range(PLAYER_RADIUS + 1.0, MAP_HEIGHT - PLAYER_RADIUS - 1.0);

            let mut collides = false;

            for (_, player_cell) in &self.players {
                if distance_between(x, y, player_cell.borrow().x, player_cell.borrow().y) <
                   2.0 * PLAYER_RADIUS {
                    collides = true;
                    break;
                }
            }

            for (_, bullet) in &self.bullets {
                if distance_between(x, y, bullet.x, bullet.y) < PLAYER_RADIUS + BULLET_RADIUS {
                    collides = true;
                    break;
                }
            }

            if !collides {
                return (x, y);
            }
        }
        println!("Failed to find a random empty spot for player after {} iterations",
                 MAX_ITERATIONS);

        (rng.gen_range(0.0, MAP_WIDTH), rng.gen_range(0.0, MAP_HEIGHT))
    }
}
