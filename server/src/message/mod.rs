//! All communication is done via discrete messages, each having a type and zero or more key-value properties.
//! Three definitions below: protocol in abstract terms, encoding of the messages (currently JSON) and transport/framing (currently TCP/WebSocket).
//! Ping/pong and timeouts are handled in the transport, so there shouldn't be any messages doing that in the protocol.
//!
//! I'm including some data bits that aren't necessarily needed, but will make tuning the game logic easier
//! (because we won't have to change the client as well as the server), this can go but it shouldn't be too much of an issue —
//!  all of those things will be constants to begin with.
//!
//! All speed/position values are in the same scale. Distance unit is the same as in HTML5 canvas, i.e. pixels. Time unit is a second.
//! Player vehicles are assumed to be squares, and the size is the square's side. Player bullets are assumed to be circles,
//! and the size is the circle's radius.
//!
//! See [#2](https://github.com/LoungeCPP/Tatsoryk/issues/2) for discussion.
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
//! * contains values of types differing from the specification, or
//! * doesn't decode properly (or violates JSON specification in any other way)
//!
//! All malformed messages MUST be rejected.

mod err;
mod player_bullet;

use std::str::FromStr;
use std::collections::BTreeMap;
use serde;
use serde_json;

pub use self::err::*;
pub use self::player_bullet::*;

#[cfg(test)]
mod tests;

/// Representation of discrete messages used for communication with the client.
///
/// Refer to the module-level documentation for more.
///
/// # Examples
///
/// Serialising a message for sending to a client:
///
/// ```
/// # let (id, x, y) = (0, 0, 0);
/// let message = Message::PlayerSpawned{
///     id: id,
///     x: x,
///     y: y,
/// }
/// let to_send = message.to_string();
/// ```
///
/// Deserialising a message received from a client:
///
/// ```
/// let msg_text = r#"{"type": "stop_moving"}"#.to_string();  // example
/// match str::parse(&msg_text) {
///     Ok(message: Message) => println!("Great! Message correct!"),
///     Err(error) => println!("Message malformed: {:?}", error),
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// **welcome** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **welcome** — sent by the server to a client, after the client successfully connects (what that means is defined by the transport) —
    ///               all data values apply to all players and are constant
    /// - `id` (u32) — server-assigned ID of the player, MUST NOT change during the connection
    /// - `speed` (f32) — speed of movement of player ships
    /// - `size` (f32) — size of the player vehicle
    /// - `bullet_speed` (f32) — speed of movement of player bullets
    /// - `bullet_size` (f32) — size of the player bullets
    Welcome {
        id: u32,
        speed: f32,
        size: f32,
        bullet_speed: f32,
        bullet_size: f32,
    },
    /// **go_away** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **go_away** — sent by the server if it rejects/terminates client connection for any reason
    /// - `reason` (str) — a message to be displayed to the user
    GoAway {
        reason: String,
    },
    /// **player_joined** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **player_joined** — sent by the server to all connected clients when a new player joins the game.
    /// - `id` (u32) — server-assigned ID of the player
    PlayerJoined {
        id: u32,
    },
    /// **player_left** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **player_left** — sent by the server to all connected clients when a player disconnects
    /// - `id` (u32) — ID of the player that just left; server MAY recycle this ID, and client MUST be ready for that
    PlayerLeft {
        id: u32,
    },
    /// **shots_fired** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **shots_fired** — sent by the server to all connected clients when a player fires a bullet
    ///                   (I'm giving bullets their own ID to make them easier to despawn but honestly not sure if that's the best of ideas)
    /// - `id` (u32) — ID of the shooting player
    /// - `bullet_id` (u32) — ID of the bullet; server MAY recycle this ID, and client MUST be ready for that
    /// - `x` (f32) — position X of the player at the moment of firing (center)
    /// - `y` (f32) — position Y of the player at the moment of firing (center)
    /// - `aim_x` (f32) — player's aiming vector X at the moment of firing
    /// - `aim_y` (f32) — player's aiming direction vector Y at the moment of firing
    ///                   (aiming direction vector MUST be normalised, i.e. its magnitude MUST be equal to 1)
    ShotsFired {
        id: u32,
        bullet_id: u32,
        x: f32,
        y: f32,
        aim_x: f32,
        aim_y: f32,
    },
    /// **player_spawned** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **player_spawned** — sent by the server to all connected clients when a player (re)spawns on the map
    /// - `id` (u32) — ID of the player
    /// - `x` (f32) — position X of the player vehicle (center)
    /// - `y` (f32) — position Y of the player vehicle (center)
    PlayerSpawned {
        id: u32,
        x: f32,
        y: f32,
    },
    /// **player_destroyed** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
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
    /// **player_moving** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **player_moving** — sent by the server to all connected clients when a player starts moving
    /// - `id` (u32) — ID of the player
    /// - `x` (f32) — position X of the player when they started to move (center)
    /// - `y` (f32) — position Y of the player when they started to move (center)
    /// - `move_x` (f32) — player's movement vector X
    /// - `move_y` (f32) — player's movement vector Y (movement vector MUST be normalised)
    PlayerMoving {
        id: u32,
        x: f32,
        y: f32,
        move_x: f32,
        move_y: f32,
    },
    /// **player_stopped** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **player_stopped** — sent by the server to all connected clients when a player stops moving
    /// - `id` (u32) — ID of the player
    /// - `x` (f32) — final position X of the player (center)
    /// - `y` (f32) — final position Y of the player (center)
    PlayerStopped {
        id: u32,
        x: f32,
        y: f32,
    },
    /// **world_state** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **world_state** — full update of the world, sent by the server to all connected clients periodically (interval up to the implementation)
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
    ///   - `move_y` (f32) — current movement direction vector Y of the bullet
    ///                      (movement direction vectors MUST be normalised, i.e. their magnitude MUST be equal to 1)
    WorldState {
        player_count: u32,
        alive_players: Vec<Player>,
        alive_bullets: Vec<Bullet>,
    },
    /// **start_moving** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **start_moving** — sent by the client to the server when the player wants to start moving or change its movement direction
    ///                    (i.e. presses/releases one or more movement keys, as long as at least one of them is still held)
    /// - `move_x` (f32) — player's movement vector X
    /// - `move_y` (f32) — player's movement vector Y
    /// (movement vector SHOULD be normalised, but the server MUST NOT assume that it is)
    StartMoving {
        move_x: f32,
        move_y: f32,
    },
    /// **stop_moving** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **stop_moving** — sent by the client to the server when the player wants to stop moving (i.e. releases held movement keys)
    StopMoving,
    /// **fire** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec)
    ///
    /// **fire** — sent by the client to the server when the player wants to fire (i.e. presses the mouse button)
    /// - `move_x` (f32) — player's aiming vector X
    /// - `move_y` (f32) — player's aiming direction vector Y (aiming direction vector SHOULD be normalised, but the server MUST NOT assume that it is)
    Fire {
        move_x: f32,
        move_y: f32,
    },
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let mut values = BTreeMap::new();
        let msg_type = match self {
            &Message::Welcome { id, speed, size, bullet_speed, bullet_size } => {
                add_data_id_speeds_sizes_entries(&mut values,
                                                 id,
                                                 speed,
                                                 size,
                                                 bullet_speed,
                                                 bullet_size);
                "welcome"
            }
            &Message::GoAway { ref reason } => {
                add_data_entry(&mut values, "reason", &reason);
                "go_away"
            }
            &Message::PlayerJoined { id } => {
                add_data_entry(&mut values, "id", &id);
                "player_joined"
            }
            &Message::PlayerLeft { id } => {
                add_data_entry(&mut values, "id", &id);
                "player_left"
            }
            &Message::ShotsFired { id, bullet_id, x, y, aim_x, aim_y } => {
                add_shot_data_entries(&mut values, id, bullet_id, x, y, aim_x, aim_y);
                "shots_fired"
            }
            &Message::PlayerSpawned { id, x, y } => {
                add_data_id_pos_entries(&mut values, id, x, y);
                "player_spawned"
            }
            &Message::PlayerDestroyed { id, killer_id, bullet_id } => {
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
            &Message::PlayerMoving { id, x, y, move_x, move_y } => {
                add_data_id_pos_moves_entries(&mut values, id, x, y, move_x, move_y);
                "player_moving"
            }
            &Message::PlayerStopped { id, x, y } => {
                add_data_id_pos_entries(&mut values, id, x, y);
                "player_stopped"
            }
            &Message::WorldState { player_count, ref alive_players, ref alive_bullets } => {
                add_data_entry(&mut values, "player_count", &player_count);
                add_data_entry(&mut values,
                               "alive_players",
                               &alive_players.iter().map(|ref p| p.to_json()).collect::<Vec<_>>());
                add_data_entry(&mut values,
                               "alive_bullets",
                               &alive_bullets.iter().map(|ref b| b.to_json()).collect::<Vec<_>>());
                "world_state"
            }
            &Message::StartMoving { move_x, move_y } => {
                add_data_move_entries(&mut values, move_x, move_y);
                "start_moving"
            }
            &Message::StopMoving => "stop_moving",
            &Message::Fire { move_x, move_y } => {
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

impl FromStr for Message {
    type Err = MessageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let json: serde_json::Value = try!(serde_json::from_str(s));

        match json.as_object() {
            Some(msg) => {
                let msg_type = try!(match msg.get("type") {
                    None => Err(MessageError::PropertyMissing(r#"Top-level Object doesn't have "type""#.to_string())),
                    Some(msg_type) => {
                        match msg_type {
                            &serde_json::Value::String(ref msg_type) => Ok(msg_type),
                            _ => {
                                Err(MessageError::BadType(r#"Message type not String"#.to_string()))
                            }
                        }
                    }
                });

                let keys = msg.keys().collect::<Vec<_>>();
                if msg_type == "stop_moving" {
                    if keys != vec!["data", "type"] && keys != vec!["type"] {
                        return Err(MessageError::PropertyMissing(format!(r#"Top-level Object is a mismatch for `{{"type"[, "data"]}}`: {:?}"#, keys)));
                    }
                } else if keys != vec!["data", "type"] {
                    return Err(MessageError::PropertyMissing(format!(r#"Top-level Object is a mismatch for `{{"type", "data"}}`: {:?}"#, keys)));
                }

                match msg.get("data") {
                    None => {
                        if msg_type == "stop_moving" {
                            Ok(Message::StopMoving)
                        } else {
                            Err(MessageError::PropertyMissing(r#"Top-level Object doesn't have "data""#.to_string()))
                        }
                    }
                    Some(data) => {
                        match data.as_object() {
                            None => {
                                Err(MessageError::BadType(r#"Top-level "data" not an Object"#
                                                              .to_string()))
                            }
                            Some(data) => {
                                if msg_type == "stop_moving" && !data.is_empty() {
                                    return Err(MessageError::ExtraneousProperty(r#"Non-empty "data" for dataless message"#.to_string()));
                                }

                                match &msg_type[..] {
                                    "welcome" => {
                                        let (id, speed, size, bullet_speed, bullet_size) =
                                            try!(decompose_stats(&data));
                                        Ok(Message::Welcome {
                                            id: id,
                                            speed: speed,
                                            size: size,
                                            bullet_speed: bullet_speed,
                                            bullet_size: bullet_size,
                                        })
                                    }
                                    "go_away" => {
                                        Ok(Message::GoAway {
                                            reason: try!(decompose_reason(&data)),
                                        })
                                    }
                                    "player_joined" => {
                                        Ok(Message::PlayerJoined { id: try!(decompose_id(&data)) })
                                    }
                                    "player_left" => {
                                        Ok(Message::PlayerLeft { id: try!(decompose_id(&data)) })
                                    }
                                    "shots_fired" => {
                                        let (id, bullet_id, x, y, aim_x, aim_y) =
                                            try!(decompose_shot(&data));
                                        Ok(Message::ShotsFired {
                                            id: id,
                                            bullet_id: bullet_id,
                                            x: x,
                                            y: y,
                                            aim_x: aim_x,
                                            aim_y: aim_y,
                                        })
                                    }
                                    "player_spawned" => {
                                        let (id, x, y) = try!(decompose_id_pos(&data));
                                        Ok(Message::PlayerSpawned {
                                            id: id,
                                            x: x,
                                            y: y,
                                        })
                                    }
                                    "player_destroyed" => {
                                        let (id, killer_id, bullet_id) =
                                            try!(decompose_destruction(&data));
                                        Ok(Message::PlayerDestroyed {
                                            id: id,
                                            killer_id: killer_id,
                                            bullet_id: bullet_id,
                                        })
                                    }
                                    "player_moving" => {
                                        let (id, x, y, move_x, move_y) =
                                            try!(decompose_id_pos_moves(&data));
                                        Ok(Message::PlayerMoving {
                                            id: id,
                                            x: x,
                                            y: y,
                                            move_x: move_x,
                                            move_y: move_y,
                                        })
                                    }
                                    "player_stopped" => {
                                        let (id, x, y) = try!(decompose_id_pos(&data));
                                        Ok(Message::PlayerStopped {
                                            id: id,
                                            x: x,
                                            y: y,
                                        })
                                    }
                                    "world_state" => {
                                        let (player_count, alive_players, alive_bullets) =
                                            try!(decompose_world_state(&data));
                                        Ok(Message::WorldState {
                                            player_count: player_count,
                                            alive_players: alive_players,
                                            alive_bullets: alive_bullets,
                                        })
                                    }
                                    "start_moving" => {
                                        let (move_x, move_y) = try!(decompose_moves(&data));
                                        Ok(Message::StartMoving {
                                            move_x: move_x,
                                            move_y: move_y,
                                        })
                                    }
                                    "stop_moving" => Ok(Message::StopMoving),
                                    "fire" => {
                                        let (move_x, move_y) = try!(decompose_moves(&data));
                                        Ok(Message::Fire {
                                            move_x: move_x,
                                            move_y: move_y,
                                        })
                                    }
                                    msg_type => Err(MessageError::BadType(format!(r#"Expected any of {:?}, got: {:?}"#,
                                                                          vec!["welcome", "go_away", "player_joined", "player_left",
                                                                               "shots_fired", "player_spawned", "player_destroyed", "player_moving",
                                                                               "player_stopped", "world_state", "start_moving", "stop_moving", "fire"],
                                                                          msg_type))),
                                }
                            }
                        }
                    }
                }
            }
            None => Err(MessageError::BadType("Top-level JSON not an Object".to_string())),
        }
    }
}

fn add_data_id_speeds_sizes_entries(data: &mut BTreeMap<String, serde_json::Value>,
                                    id: u32,
                                    speed: f32,
                                    size: f32,
                                    bullet_speed: f32,
                                    bullet_size: f32) {
    add_data_entry(data, "id", &id);
    add_data_entry(data, "speed", &speed);
    add_data_entry(data, "size", &size);
    add_data_entry(data, "bullet_speed", &bullet_speed);
    add_data_entry(data, "bullet_size", &bullet_size);
}

fn add_data_id_pos_moves_entries(data: &mut BTreeMap<String, serde_json::Value>,
                                 id: u32,
                                 x: f32,
                                 y: f32,
                                 move_x: f32,
                                 move_y: f32) {
    add_data_id_pos_entries(data, id, x, y);
    add_data_move_entries(data, move_x, move_y);
}

fn add_data_id_pos_entries(data: &mut BTreeMap<String, serde_json::Value>,
                           id: u32,
                           x: f32,
                           y: f32) {
    add_data_entry(data, "id", &id);
    add_data_entry(data, "x", &x);
    add_data_entry(data, "y", &y);
}

fn add_data_move_entries(data: &mut BTreeMap<String, serde_json::Value>,
                         move_x: f32,
                         move_y: f32) {
    add_data_entry(data, "move_x", &move_x);
    add_data_entry(data, "move_y", &move_y);
}

fn add_shot_data_entries(data: &mut BTreeMap<String, serde_json::Value>,
                         id: u32,
                         bullet_id: u32,
                         x: f32,
                         y: f32,
                         aim_x: f32,
                         aim_y: f32) {
    add_data_entry(data, "id", &id);
    add_data_entry(data, "bullet_id", &bullet_id);
    add_data_entry(data, "x", &x);
    add_data_entry(data, "y", &y);
    add_data_entry(data, "aim_x", &aim_x);
    add_data_entry(data, "aim_y", &aim_y);
}

fn add_data_entry<T: serde::Serialize>(data: &mut BTreeMap<String, serde_json::Value>,
                                       name: &str,
                                       what: &T) {
    let _ = data.insert(name.to_string(), serde_json::to_value(what));
}

fn decompose_moves(data: &BTreeMap<String, serde_json::Value>) -> Result<(f32, f32), MessageError> {
    try!(decompose_assert_size(data.len(), 2));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(), vec!["move_x", "move_y"]));

    Ok((try!(unpack_f32(data.get("move_x").unwrap())),
        try!(unpack_f32(data.get("move_y").unwrap()))))
}

fn decompose_id_pos(data: &BTreeMap<String, serde_json::Value>)
                    -> Result<(u32, f32, f32), MessageError> {
    try!(decompose_assert_size(data.len(), 3));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(), vec!["id", "x", "y"]));

    Ok((try!(unpack_u32(data.get("id").unwrap())),
        try!(unpack_f32(data.get("x").unwrap())),
        try!(unpack_f32(data.get("y").unwrap()))))
}

fn decompose_stats(data: &BTreeMap<String, serde_json::Value>)
                   -> Result<(u32, f32, f32, f32, f32), MessageError> {
    try!(decompose_assert_size(data.len(), 5));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(),
                               vec!["bullet_size", "bullet_speed", "id", "size", "speed"]));

    Ok((try!(unpack_u32(data.get("id").unwrap())),
        try!(unpack_f32(data.get("speed").unwrap())),
        try!(unpack_f32(data.get("size").unwrap())),
        try!(unpack_f32(data.get("bullet_speed").unwrap())),
        try!(unpack_f32(data.get("bullet_size").unwrap()))))
}

fn decompose_reason(data: &BTreeMap<String, serde_json::Value>) -> Result<String, MessageError> {
    try!(decompose_assert_size(data.len(), 1));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(), vec!["reason"]));

    Ok(try!(unpack_str(data.get("reason").unwrap())))
}

fn decompose_id(data: &BTreeMap<String, serde_json::Value>) -> Result<u32, MessageError> {
    try!(decompose_assert_size(data.len(), 1));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(), vec!["id"]));

    Ok(try!(unpack_u32(data.get("id").unwrap())))
}

fn decompose_shot(data: &BTreeMap<String, serde_json::Value>)
                  -> Result<(u32, u32, f32, f32, f32, f32), MessageError> {
    try!(decompose_assert_size(data.len(), 6));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(),
                               vec!["aim_x", "aim_y", "bullet_id", "id", "x", "y"]));

    Ok((try!(unpack_u32(data.get("id").unwrap())),
        try!(unpack_u32(data.get("bullet_id").unwrap())),
        try!(unpack_f32(data.get("x").unwrap())),
        try!(unpack_f32(data.get("y").unwrap())),
        try!(unpack_f32(data.get("aim_x").unwrap())),
        try!(unpack_f32(data.get("aim_y").unwrap()))))
}

fn decompose_destruction(data: &BTreeMap<String, serde_json::Value>)
                         -> Result<(u32, Option<u32>, Option<u32>), MessageError> {
    match data.len() {
        1 => Ok((try!(decompose_id(data)), None, None)),
        3 => {
            try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(),
                                       vec!["bullet_id", "id", "killer_id"]));

            Ok((try!(unpack_u32(data.get("id").unwrap())),
                Some(try!(unpack_u32(data.get("killer_id").unwrap()))),
                Some(try!(unpack_u32(data.get("bullet_id").unwrap())))))
        }
        len => {
            if len > 3 {
                Err(MessageError::ExtraneousProperty(format!(r#"Expected 1 or 3, got {}"#, len)))
            } else {
                Err(MessageError::PropertyMissing(format!(r#"Expected 1 or 3, got {}"#, len)))
            }
        }
    }
}

fn decompose_world_state(data: &BTreeMap<String, serde_json::Value>)
                         -> Result<(u32, Vec<Player>, Vec<Bullet>), MessageError> {
    try!(decompose_assert_size(data.len(), 3));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(),
                               vec!["alive_bullets", "alive_players", "player_count"]));

    let alive_players = try!(unpack_from_jsonnable(try!(unpack_arr(data.get("alive_players")
                                                                       .unwrap())),
                                                   Player::from_json,
                                                   Player::not_moving(0, 0f32, 0f32)));
    let alive_bullets = try!(unpack_from_jsonnable(try!(unpack_arr(data.get("alive_bullets")
                                                                       .unwrap())),
                                                   Bullet::from_json,
                                                   Bullet::not_moving(0, 0f32, 0f32)));

    Ok((try!(unpack_u32(data.get("player_count").unwrap())), alive_players, alive_bullets))
}

fn decompose_id_pos_moves(data: &BTreeMap<String, serde_json::Value>)
                          -> Result<(u32, f32, f32, f32, f32), MessageError> {
    try!(decompose_assert_size(data.len(), 5));
    try!(decompose_assert_keys(data.keys().collect::<Vec<_>>(),
                               vec!["id", "move_x", "move_y", "x", "y"]));

    Ok((try!(unpack_u32(data.get("id").unwrap())),
        try!(unpack_f32(data.get("x").unwrap())),
        try!(unpack_f32(data.get("y").unwrap())),
        try!(unpack_f32(data.get("move_x").unwrap())),
        try!(unpack_f32(data.get("move_y").unwrap()))))
}

fn decompose_assert_size(len: usize, expected: usize) -> Result<(), MessageError> {
    if len > expected {
        return Err(MessageError::ExtraneousProperty(format!(r#"Expected {}, got {}"#,
                                                            expected,
                                                            len)));
    } else if len < expected {
        return Err(MessageError::PropertyMissing(format!(r#"Expected {}, got {}"#, expected, len)));
    } else {
        Ok(())
    }
}

fn decompose_assert_keys(keys: Vec<&String>,
                         expected: Vec<&'static str>)
                         -> Result<(), MessageError> {
    if keys != expected {
        return Err(MessageError::ExtraneousProperty(format!(r#"Data Object is a mismatch for {:?}: {:?}"#, expected, keys)));
    } else {
        Ok(())
    }
}

fn unpack_f32(val: &serde_json::Value) -> Result<f32, MessageError> {
    match val {
        &serde_json::Value::F64(f) => Ok(f as f32),
        &serde_json::Value::I64(i) => Ok(i as f32),
        &serde_json::Value::U64(u) => Ok(u as f32),
        _ => Err(MessageError::BadType("Expected f32-compatible type".to_string())),
    }
}

fn unpack_u32(val: &serde_json::Value) -> Result<u32, MessageError> {
    match val {
        &serde_json::Value::I64(i) => Ok(i as u32),
        &serde_json::Value::U64(u) => Ok(u as u32),
        _ => Err(MessageError::BadType("Expected u32-compatible type".to_string())),
    }
}

fn unpack_str(val: &serde_json::Value) -> Result<String, MessageError> {
    match val {
        &serde_json::Value::String(ref s) => Ok(s.clone()),
        _ => Err(MessageError::BadType("Expected String".to_string())),
    }
}

fn unpack_arr<'v>(val: &'v serde_json::Value) -> Result<&'v Vec<serde_json::Value>, MessageError> {
    match val {
        &serde_json::Value::Array(ref s) => Ok(s),
        _ => Err(MessageError::BadType("Expected Array".to_string())),
    }
}

fn unpack_from_jsonnable<T: Copy, F: Fn(&serde_json::Value) -> Result<T, MessageError>>
    (vals: &Vec<serde_json::Value>,
     from_json: F,
     placeholder: T)
     -> Result<Vec<T>, MessageError> {
    let mut err: Option<MessageError> = None;
    let alive_players = vals.iter()
                            .map(|ast| {
                                if err.is_none() {
                                    match from_json(ast) {
                                        Err(error) => {
                                            err = Some(error);
                                            Ok(placeholder)
                                        }
                                        ok => ok,
                                    }
                                } else {
                                    Ok(placeholder)
                                }
                            })
                            .collect::<Vec<_>>();
    if let Some(err) = err {
        return Err(err);
    }
    Ok(alive_players.into_iter().map(Result::unwrap).collect::<Vec<_>>())
}
