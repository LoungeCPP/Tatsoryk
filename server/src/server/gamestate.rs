//! The state of the game itself.

use message;

use std::collections::HashMap;
use std::sync::mpsc;

use math::distance_between;
use rand::{thread_rng, Rng};

use self::super::Client;
use self::super::WebSocketEvent;

static BULLET_RADIUS: f32 = 5.0;
static PLAYER_RADIUS: f32 = 10.0;
static BULLET_SPEED: f32 = 3.0;
static PLAYER_SPEED: f32 = 2.0;
static MAP_HEIGHT: f32 = 500.0;
static MAP_WIDTH: f32 = 500.0;

/// The `GameState` contains the whole state of the game.
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
        let player_ids: Vec<_> = self.players.keys().map(|i| *i).collect();
        for cur_player_id in &player_ids {
            {
                let cur_player = self.players.get(cur_player_id).unwrap();
                match (cur_player.move_x, cur_player.move_y) {
                    (None, None) => continue,
                    (Some(move_x), Some(move_y)) => {
                        let mut collides = false;
                        for cmp_player_id in &player_ids {
                            if cmp_player_id != cur_player_id {
                                let cmp_player = self.players.get(cmp_player_id).unwrap();
                                if distance_between(cur_player.x + move_x,
                                                    cur_player.y + move_y,
                                                    cmp_player.x,
                                                    cmp_player.y) <
                                   2.0 * PLAYER_RADIUS {
                                    collides = true;
                                    break;
                                }
                            }
                        }
                        if collides {
                            continue;
                        }
                    }
                    _ => panic!("Invariant not met: player moves only in one direction"),
                }
            }

            let mut player = self.players.get_mut(cur_player_id).unwrap();
            player.x = (player.x + player.move_x.unwrap_or(0.0) * PLAYER_SPEED)
                           .max(PLAYER_RADIUS)
                           .min(MAP_WIDTH - PLAYER_RADIUS);
            player.y = (player.y + player.move_y.unwrap_or(0.0) * PLAYER_SPEED)
                           .max(PLAYER_RADIUS)
                           .min(MAP_HEIGHT - PLAYER_RADIUS);
        }

        let mut destroyed_bullets = Vec::new();
        let mut destroyed_players = Vec::new();

        for (_, bullet) in &mut self.bullets {
            bullet.x += bullet.move_x.unwrap_or(0.0) * BULLET_SPEED;
            bullet.y += bullet.move_y.unwrap_or(0.0) * BULLET_SPEED;

            if bullet.x < 0.0 || bullet.x > MAP_WIDTH || bullet.y < 0.0 || bullet.y > MAP_HEIGHT {
                destroyed_bullets.push(bullet.id);
            }
        }

        // Check for collisions
        for (_, bullet) in &mut self.bullets {
            for (_, player) in &mut self.players {
                if distance_between(bullet.x, bullet.y, player.x, player.y) <
                   BULLET_RADIUS + PLAYER_RADIUS {
                    destroyed_bullets.push(bullet.id);
                    destroyed_players.push(player.id);
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
            let dead_player = self.players.get_mut(&player_id).unwrap();
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
                    speed: PLAYER_SPEED,
                    size: PLAYER_RADIUS,
                    bullet_speed: BULLET_SPEED,
                    bullet_size: BULLET_RADIUS,
                };

                let _ = client.send(welcome_message.to_string());
                let _ = client.send(self.serialize());

                let (x, y) = self.random_free_spot(&mut thread_rng());
                let _ = self.players
                            .insert(client.id, message::Player::not_moving(client.id, x, y));

                let _ = self.clients.insert(client.id, client);
            }
            WebSocketEvent::ClientClosed { client_id } => {
                let _ = self.players.remove(&client_id);
                let _ = self.clients.remove(&client_id);
            }
            WebSocketEvent::ClientMessage { client_id, message } => {
                self.process_client_message(client_id, message);
            }
        }
    }

    /// Serialize the entire game state into one json string.
    fn serialize(&self) -> String {
        let players: Vec<_> = self.players
                                  .values()
                                  .cloned()
                                  .collect();
        let bullets: Vec<_> = self.bullets
                                  .values()
                                  .cloned()
                                  .collect();
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
                let player = self.players.get_mut(&client_id).unwrap();
                player.move_x = Some(move_x);
                player.move_y = Some(move_y);
            }
            message::Message::StopMoving => {
                let player = self.players.get_mut(&client_id).unwrap();
                player.move_x = None;
                player.move_y = None;
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

    fn random_free_spot<R: Rng>(&self, rng: &mut R) -> (f32, f32) {
        static MAX_ITERATIONS: u32 = 100;

        let min_vial_x = PLAYER_RADIUS;
        let min_vial_y = PLAYER_RADIUS;
        let max_vial_x = MAP_WIDTH - PLAYER_RADIUS;
        let max_vial_y = MAP_HEIGHT - PLAYER_RADIUS;

        for _ in 1..MAX_ITERATIONS {
            let x: f32 = rng.gen_range(min_vial_x, max_vial_x);
            let y: f32 = rng.gen_range(min_vial_y, max_vial_y);

            let mut collides = false;

            for (_, player) in &self.players {
                if distance_between(x, y, player.x, player.y) < 2.0 * PLAYER_RADIUS {
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
