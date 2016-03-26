//! All communication is done via discrete messages, each having a type and zero or more key-value properties.
//! Three definitions below: protocol in abstract terms, encoding of the messages (currently JSON) and transport/framing (currently TCP/WebSocket).
//! Ping/pong and timeouts are handled in the transport, so there shouldn't be any messages doing that in the protocol.
//!
//! Version/feature negotiation can be added in the future if needed.
//! Client and server is assumed to be always talking the same iteration of the protocol, and the behaviour is undefined otherwise.
//!
//! I'm including some data bits that aren't necessarily needed, but will make tuning the game logic easier
//! (because we won't have to change the client as well as the server),
//! this can go but it shouldn't be too much of an issue — all of those things will be constants to begin with.
//!
//! All speed/position values are in the same scale, but the scale is up to be determined. Could be pixels (and pixels/second for speed) for now.
//!
//! I'm ignoring differences in viewport sizes, in future this should be addressed somehow so players with bigger screens don't get an advantage.
//!
//! I didn't include aiming updates, we can probably live without rendering other players' aim vectors.
//!
//!
//! # Encoding (JSON)
//!
//! To start with, we encode all messages as JSON objects, with the type ID being stored in `type` key,
//! and message properties being stored as another object in `data` key (and that object has key per property), e.g.
//!
//! ```json
//! {
//!     "type": "world_state",
//!     "data": {
//!         "player_count": 32,
//!         "alive_players": [
//!             { "id": 1, "x": 34.66, "y": 21.44 },
//!             { "id": 6, "x": 67.34, "y": 22.22 }
//!         ]
//!     }
//! }
//! ```
//!
//! The `data` key MAY be omitted if the message doesn't define any properties.
//! Optional properties MUST be omitted if they're not present (and not set to `null`).
//!
//! Newlines and indenting added for example purposes: all the exchanged messages SHOULD NOT contain any unnecessary whitespace.
//!
//! A message is malformed if it:
//! * contains unknown `type` value, or
//! * contains extra fields, or
//! * doesn't contain any required fields, or
//! * doesn't decode properly (or violates JSON specification in any other way)
//!
//! All malformed messages MUST be rejected.

use std::collections::BTreeMap;
use serde;
use serde_json;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// **welcome** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **welcome** — sent by the server to a client, after the client successfully connects (what that means is defined by the transport)
    /// - `id` (u32) — server-assigned ID of the player, MUST NOT change during the connection
    /// - `speed` (f32) — speed of movement of player's entity
    /// - `fire_speed` (f32) — speed of movement of player's bullets
    Welcome {
        id: u32,
        speed: f32,
        fire_speed: f32,
    },
    /// **go_away** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **go_away** — sent by the server if it rejects/terminates client connection for any reason
    /// - `reason` (str) — a message to be displayed to the user
    GoAway {
        reason: String,
    },
    /// **player_joined** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **player_joined** — sent by the server to all connected clients when a new player joins the game.
    /// - `id` (u32) — server-assigned ID of the player
    /// - `speed` (f32) — speed of movement of player's entity
    /// - `fire_speed` (f32) — speed of movement of player's bullets
    ///                        (assumed to be constant throughout the game:
    ///                         this can be moved to `shots_fired` if we decide to make switcheable weapons or something)
    PlayerJoined {
        id: u32,
        speed: f32,
        fire_speed: f32,
    },
    /// **player_left** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **player_left** — sent by the server to all connected clients when a player disconnects
    /// - `id` (u32) — ID of the player that just left; server MAY recycle this ID, and client MUST be ready for that
    PlayerLeft {
        id: u32,
    },
    /// **shots_fired** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **shots_fired** — sent by the server to all connected clients when a player fires a bullet
    ///                   (I'm giving bullets their own ID to make them easier to despawn but honestly not sure if that's the best of ideas)
    /// - `id` (u32) — ID of the shooting player
    /// - `bullet_id` (u32) — ID of the bullet; server MAY recycle this ID, and client MUST be ready for that
    /// - `x` (f32) — position X of the player at the moment of firing (center)
    /// - `y` (f32) — position Y of the player at the moment of firing (center)
    /// - `aim_x` (f32) — player's aiming vector X at the moment of firing
    /// - `aim_y` (f32) — player's aiming vector Y at the moment of firing
    ///
    /// (aiming vector MUST be normalised)
    ShotsFired {
        id: u32,
        speed: f32,
        fire_speed: f32,
    },
    /// **player_spawned** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **player_spawned** — sent by the server to all connected clients when a player (re)spawns on the map
    /// - `id` (u32) — ID of the player
    /// - `x` (f32) — position X of the entity (center)
    /// - `y` (f32) — position Y of the entity (center)
    PlayerSpawned {
        id: u32,
        x: f32,
        y: f32,
        move_x: f32,
        move_y: f32,
    },
    /// **player_destroyed** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **player_destroyed** — sent by the server to all connected clients when a player despawns from the map
    /// - `id` (u32) — ID of the player
    /// - `killer_id` (Option&lt;u32&gt;) — ID of the killer, if any
    /// - `bullet_id` (Option&lt;u32&gt;) — ID of the bullet, if any; MUST be present if `killer_id` is present
    PlayerDestroyed {
        id: u32,
        killer_id: Option<u32>,
        bullet_id: Option<u32>,
    },
    /// **player_moving** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **player_moving** — sent by the server to all connected clients when a player starts moving
    /// - `id` (u32) — ID of the player
    /// - `x` (f32) — position X of the player when they started to move (center)
    /// - `y` (f32) — position Y of the player when they started to move (center)
    /// - `move_x` (f32) — player's movement vector X
    /// - `move_y` (f32) — player's movement vector Y
    /// (movement vector MUST be normalised)
    PlayerMoving {
        id: u32,
        x: f32,
        y: f32,
        move_x: f32,
        move_y: f32,
    },
    /// **player_stopped** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **player_stopped** — sent by the server to all connected clients when a player stops moving
    /// - `id` (u32) — ID of the player
    /// - `x` (f32) — final position X of the player (center)
    /// - `y` (f32) — final position Y of the player (center)
    PlayerStopped {
        id: u32,
        x: f32,
        y: f32,
        move_x: f32,
        move_y: f32,
    },
    /// **world_state** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **world_state** — full update of the world (bullets not included because their movement vector never changes so the client should be able to render them perfectly without full update), sent by the server to all connected clients from time to time
    /// - `player_count` (u32) — count of all connected players
    /// - `alive_players` (Player[]) — an array of all currently alive players, each containing:
    ///   - `id` (u32) — ID of the player
    ///   - `x` (f32) — current position X of the player
    ///   - `y` (f32) — current position Y of the player
    ///   - `move_x` (Optional&lt;f32&gt;) — current movement vector X of the player, if player is moving
    ///   - `move_y` (Optional&lt;f32&gt;) — current movement vector Y of the player, if player is moving
    /// - `alive_bullets` (Bullet[]) — an array of all currently alive bullets, each containing:
    ///   - `id` (u32) — ID of the bullet
    ///   - `x` (f32) — current position X of the bullet
    ///   - `y` (f32) — current position Y of the bullet
    ///   - `move_x` (f32) — current movement vector X of the bullet
    ///   - `move_y` (f32) — current movement vector Y of the bullet
    /// (movement vectors MUST be normalised)
    WorldState,
    /// **start_moving** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **start_moving** — sent by the client to the server when the player wants to start moving (i.e. presses one or more movement keys)
    /// - `move_x` (f32) — player's movement vector X
    /// - `move_y` (f32) — player's movement vector Y
    /// (movement vector SHOULD be normalised, but the server MUST NOT assume that it is)
    StartMoving {
        move_x: f32,
        move_y: f32,
    },
    /// **stop_moving** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **stop_moving** — sent by the client to the server when the player wants to stop moving (i.e. releases held movement keys)
    StopMoving,
    /// **fire** message, as defined by [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2)
    ///
    /// **fire** — sent by the client to the server when the player wants to fire (i.e. presses the mouse button)
    /// - `move_x` (f32) — player's aiming vector X
    /// - `move_y` (f32) — player's aiming vector Y
    /// (aiming vector SHOULD be normalised, but the server MUST NOT assume that it is)
    Fire {
        move_x: f32,
        move_y: f32,
    },
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let mut values = BTreeMap::new();
        let msg_type = match self {
            &Message::Welcome{id, speed, fire_speed} => {
                add_data_id_speeds_entries(&mut values, id, speed, fire_speed);
                "welcome"
            }
            &Message::GoAway{ref reason} => {
                add_data_entry(&mut values, "reason", &reason);
                "go_away"
            }
            &Message::PlayerJoined{id, speed, fire_speed} => {
                add_data_id_speeds_entries(&mut values, id, speed, fire_speed);
                "player_joined"
            }
            &Message::PlayerLeft{id} => {
                add_data_entry(&mut values, "id", &id);
                "player_left"
            }
            &Message::ShotsFired{id, speed, fire_speed} => {
                add_data_id_speeds_entries(&mut values, id, speed, fire_speed);
                "shots_fired"
            }
            &Message::PlayerSpawned{id, x, y, move_x, move_y} => {
                add_data_id_pos_moves_entries(&mut values, id, x, y, move_x, move_y);
                "player_spawned"
            }
            &Message::PlayerDestroyed{id, killer_id, bullet_id} => {
                add_data_entry(&mut values, "id", &id);
                match (killer_id, bullet_id) {
                    (Some(killer_id), Some(bullet_id)) => {
                        add_data_entry(&mut values, "killer_id", &killer_id);
                        add_data_entry(&mut values, "bullet_id", &bullet_id);
                    }
                    (None, None) => {}
                    _ => panic!("killer_id and bullet_id must be either both Some or both None"),
                }
                "player_destroyed"
            }
            &Message::PlayerMoving{id, x, y, move_x, move_y} => {
                add_data_id_pos_moves_entries(&mut values, id, x, y, move_x, move_y);
                "player_moving"
            }
            &Message::PlayerStopped{id, x, y, move_x, move_y} => {
                add_data_id_pos_moves_entries(&mut values, id, x, y, move_x, move_y);
                "player_stopped"
            }
            &Message::WorldState => "world_state",  // TODO
            &Message::StartMoving{move_x, move_y} => {
                add_data_move_entries(&mut values, move_x, move_y);
                "start_moving"
            }
            &Message::StopMoving => "stop_moving",
            &Message::Fire{move_x, move_y} => {
                add_data_move_entries(&mut values, move_x, move_y);
                "fire"
            }
        };

        let mut root_obj = BTreeMap::new();
        let _ = root_obj.insert("type".to_string(),
                                serde_json::Value::String(msg_type.to_string()));
        if !values.is_empty() {
            let _ = root_obj.insert("data".to_string(), serde_json::Value::Object(values));
        }

        serde_json::to_string(&serde_json::Value::Object(root_obj)).unwrap()
    }
}

fn add_data_id_speeds_entries(data: &mut BTreeMap<String, serde_json::Value>,
                              id: u32,
                              speed: f32,
                              fire_speed: f32) {
    add_data_entry(data, "id", &id);
    add_data_entry(data, "speed", &speed);
    add_data_entry(data, "fire_speed", &fire_speed);
}

fn add_data_id_pos_moves_entries(data: &mut BTreeMap<String, serde_json::Value>,
                                 id: u32,
                                 x: f32,
                                 y: f32,
                                 move_x: f32,
                                 move_y: f32) {
    add_data_entry(data, "id", &id);
    add_data_entry(data, "x", &x);
    add_data_entry(data, "y", &y);
    add_data_move_entries(data, move_x, move_y);
}

fn add_data_move_entries(data: &mut BTreeMap<String, serde_json::Value>,
                         move_x: f32,
                         move_y: f32) {
    add_data_entry(data, "move_x", &move_x);
    add_data_entry(data, "move_y", &move_y);
}

fn add_data_entry<T: serde::Serialize>(data: &mut BTreeMap<String, serde_json::Value>,
                                       name: &str,
                                       what: &T) {
    let _ = data.insert(name.to_string(), serde_json::to_value(what));
}


#[cfg(test)]
mod tests {
    extern crate rand;

    use std::iter::FromIterator;
    use std::collections::BTreeMap;
    use self::super::Message;
    use self::rand::{Rng, thread_rng};
    use serde_json::{self, Value};

    #[test]
    fn welcome_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let speed = gen_f32(&mut rng);
        let fire_speed = gen_f32(&mut rng);

        let json_txt = Message::Welcome {
                           id: id,
                           speed: speed,
                           fire_speed: fire_speed,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("welcome".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("speed".to_string(), Value::F64(speed as f64)),
                    ("fire_speed".to_string(), Value::F64(fire_speed as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn go_away_serializes_properly() {
        let mut rng = thread_rng();
        let reason: String = {
            let len = rng.gen_range(1, 100);
            rng.gen_ascii_chars().take(len).collect()
        };

        let json_txt = Message::GoAway { reason: reason.clone() }.to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("go_away".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("reason".to_string(), Value::String(reason.clone())),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn player_joined_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let speed = gen_f32(&mut rng);
        let fire_speed = gen_f32(&mut rng);

        let json_txt = Message::PlayerJoined {
                           id: id,
                           speed: speed,
                           fire_speed: fire_speed,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_joined".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("speed".to_string(), Value::F64(speed as f64)),
                    ("fire_speed".to_string(), Value::F64(fire_speed as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn player_left_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();

        let json_txt = Message::PlayerLeft { id: id }.to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_left".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn shots_fired_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let speed = gen_f32(&mut rng);
        let fire_speed = gen_f32(&mut rng);

        let json_txt = Message::ShotsFired {
                           id: id,
                           speed: speed,
                           fire_speed: fire_speed,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("shots_fired".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("speed".to_string(), Value::F64(speed as f64)),
                    ("fire_speed".to_string(), Value::F64(fire_speed as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn player_spawned_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);
        let move_x = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        let json_txt = Message::PlayerSpawned {
                           id: id,
                           x: x,
                           y: y,
                           move_x: move_x,
                           move_y: move_y,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_spawned".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("x".to_string(), Value::F64(x as f64)),
                    ("y".to_string(), Value::F64(y as f64)),
                    ("move_x".to_string(), Value::F64(move_x as f64)),
                    ("move_y".to_string(), Value::F64(move_y as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn player_destroyed_no_killer_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();

        let json_txt = Message::PlayerDestroyed {
                           id: id,
                           killer_id: None,
                           bullet_id: None,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_destroyed".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn player_destroyed_with_killer_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let killer_id: u32 = rng.gen();
        let bullet_id: u32 = rng.gen();

        let json_txt = Message::PlayerDestroyed {
                           id: id,
                           killer_id: Some(killer_id),
                           bullet_id: Some(bullet_id),
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_destroyed".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("killer_id".to_string(), Value::U64(killer_id as u64)),
                    ("bullet_id".to_string(), Value::U64(bullet_id as u64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    #[should_panic]
    fn player_destroyed_with_killer_no_bullet_panics() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let killer_id: u32 = rng.gen();

        let _ = Message::PlayerDestroyed {
                    id: id,
                    killer_id: Some(killer_id),
                    bullet_id: None,
                }
                .to_string();
    }

    #[test]
    #[should_panic]
    fn player_destroyed_with_bullet_no_killer_panics() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let bullet_id: u32 = rng.gen();

        let _ = Message::PlayerDestroyed {
                    id: id,
                    killer_id: None,
                    bullet_id: Some(bullet_id),
                }
                .to_string();
    }

    #[test]
    fn player_moving_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);
        let move_x = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        let json_txt = Message::PlayerMoving {
                           id: id,
                           x: x,
                           y: y,
                           move_x: move_x,
                           move_y: move_y,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_moving".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("x".to_string(), Value::F64(x as f64)),
                    ("y".to_string(), Value::F64(y as f64)),
                    ("move_x".to_string(), Value::F64(move_x as f64)),
                    ("move_y".to_string(), Value::F64(move_y as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn player_stopped_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);
        let move_x = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        let json_txt = Message::PlayerStopped {
                           id: id,
                           x: x,
                           y: y,
                           move_x: move_x,
                           move_y: move_y,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_stopped".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("x".to_string(), Value::F64(x as f64)),
                    ("y".to_string(), Value::F64(y as f64)),
                    ("move_x".to_string(), Value::F64(move_x as f64)),
                    ("move_y".to_string(), Value::F64(move_y as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn world_state_serializes_properly() {
        //TODO
        println!("{}", Message::WorldState.to_string());
        assert!(true);
    }

    #[test]
    fn start_moving_serializes_properly() {
        let mut rng = thread_rng();
        let move_x = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        let json_txt = Message::StartMoving {
                           move_x: move_x,
                           move_y: move_y,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("start_moving".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("move_x".to_string(), Value::F64(move_x as f64)),
                    ("move_y".to_string(), Value::F64(move_y as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn stop_moving_serializes_properly() {
        let json_txt = Message::StopMoving.to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("stop_moving".to_string())),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    #[test]
    fn fire_serializes_properly() {
        let mut rng = thread_rng();
        let move_x = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        let json_txt = Message::Fire {
                           move_x: move_x,
                           move_y: move_y,
                       }
                       .to_string();

        let expected_json = Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("fire".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("move_x".to_string(), Value::F64(move_x as f64)),
                    ("move_y".to_string(), Value::F64(move_y as f64)),
                ]
            ))),
        ]));

        assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                   expected_json);
    }

    fn gen_f32<R: Rng>(rng: &mut R) -> f32 {
        // Randoming actual floats hits us when widening them to f64
        (rng.gen_range(0u32, 99u32) as f32) + 0.5f32
    }
}
