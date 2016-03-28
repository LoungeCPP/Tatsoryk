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

use std::str::FromStr;
use std::collections::BTreeMap;
use serde;
use serde_json;

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
    WorldState,
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
            &Message::Welcome{id, speed, size, bullet_speed, bullet_size} => {
                add_data_id_speeds_sizes_entries(&mut values,
                                                 id,
                                                 speed,
                                                 size,
                                                 bullet_speed,
                                                 bullet_size);
                "welcome"
            }
            &Message::GoAway{ref reason} => {
                add_data_entry(&mut values, "reason", &reason);
                "go_away"
            }
            &Message::PlayerJoined{id} => {
                add_data_entry(&mut values, "id", &id);
                "player_joined"
            }
            &Message::PlayerLeft{id} => {
                add_data_entry(&mut values, "id", &id);
                "player_left"
            }
            &Message::ShotsFired{id, bullet_id, x, y, aim_x, aim_y} => {
                add_shot_data_entries(&mut values, id, bullet_id, x, y, aim_x, aim_y);
                "shots_fired"
            }
            &Message::PlayerSpawned{id, x, y} => {
                add_data_id_pos_entries(&mut values, id, x, y);
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
            &Message::PlayerStopped{id, x, y} => {
                add_data_id_pos_entries(&mut values, id, x, y);
                "player_stopped"
            }
            &Message::WorldState => "world_state",  //TODO
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
                // TODO: implement world_state
                if msg_type == "stop_moving" || msg_type == "world_state" {
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
                        } else if msg_type == "world_state" {
                            // TODO: implement WorldState
                            Ok(Message::WorldState)
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
                                // TODO: implement WorldState
                                if (msg_type == "stop_moving" || msg_type == "world_state") &&
                                   !data.is_empty() {
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
                                    "world_state" => Ok(Message::WorldState), // TODO
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

#[derive(Debug)]
pub enum MessageError {
    JsonError(serde_json::Error),
    PropertyMissing(String),
    ExtraneousProperty(String),
    BadType(String),
}

impl From<serde_json::Error> for MessageError {
    fn from(sje: serde_json::Error) -> Self {
        MessageError::JsonError(sje)
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


#[cfg(test)]
mod tests {
    extern crate rand;

    use std::iter::FromIterator;
    use std::collections::BTreeMap;
    use self::rand::Rng;
    use serde_json::Value;

    mod ser {
        use self::super::*;
        use self::super::rand::{Rng, thread_rng};
        use self::super::super::Message;
        use serde_json::{self, Value};

        #[test]
        fn welcome_serializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();
            let speed = gen_f32(&mut rng);
            let size = gen_f32(&mut rng);
            let bullet_speed = gen_f32(&mut rng);
            let bullet_size = gen_f32(&mut rng);

            let json_txt = Message::Welcome {
                               id: id,
                               speed: speed,
                               size: size,
                               bullet_speed: bullet_speed,
                               bullet_size: bullet_size,
                           }
                           .to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       welcome_expected_json(id, speed, size, bullet_speed, bullet_size));
        }

        #[test]
        fn go_away_serializes_properly() {
            let mut rng = thread_rng();
            let reason: String = {
                let len = rng.gen_range(1, 100);
                rng.gen_ascii_chars().take(len).collect()
            };

            let json_txt = Message::GoAway { reason: reason.clone() }.to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       go_away_expected_json(reason));
        }

        #[test]
        fn player_joined_serializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();

            let json_txt = Message::PlayerJoined { id: id }.to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       player_joined_expected_json(id));
        }

        #[test]
        fn player_left_serializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();

            let json_txt = Message::PlayerLeft { id: id }.to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       player_left_expected_json(id));
        }

        #[test]
        fn shots_fired_serializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();
            let bullet_id: u32 = rng.gen();
            let x = gen_f32(&mut rng);
            let y = gen_f32(&mut rng);
            let aim_x = gen_f32(&mut rng);
            let aim_y = gen_f32(&mut rng);

            let json_txt = Message::ShotsFired {
                               id: id,
                               bullet_id: bullet_id,
                               x: x,
                               y: y,
                               aim_x: aim_x,
                               aim_y: aim_y,
                           }
                           .to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       shots_fired_expected_json(id, bullet_id, x, y, aim_x, aim_y));
        }

        #[test]
        fn player_spawned_serializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();
            let x = gen_f32(&mut rng);
            let y = gen_f32(&mut rng);

            let json_txt = Message::PlayerSpawned {
                               id: id,
                               x: x,
                               y: y,
                           }
                           .to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       player_spawned_expected_json(id, x, y));
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

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       player_destroyed_no_killer_expected_json(id));
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


            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       player_destroyed_with_killer_expected_json(id, killer_id, bullet_id));
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

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       player_moving_expected_json(id, x, y, move_x, move_y));
        }

        #[test]
        fn player_stopped_serializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();
            let x = gen_f32(&mut rng);
            let y = gen_f32(&mut rng);

            let json_txt = Message::PlayerStopped {
                               id: id,
                               x: x,
                               y: y,
                           }
                           .to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       player_stopped_expected_json(id, x, y));
        }

        #[test]
        fn world_state_serializes_properly() {
            // TODO implement WorldState
            assert_eq!(serde_json::from_str::<Value>(&*&Message::WorldState.to_string()).unwrap(),
                       world_state_expected_json());
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

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       start_moving_expected_json(move_x, move_y));
        }

        #[test]
        fn stop_moving_serializes_properly() {
            let json_txt = Message::StopMoving.to_string();

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       stop_moving_expected_json());
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

            assert_eq!(serde_json::from_str::<Value>(&*&json_txt).unwrap(),
                       fire_expected_json(move_x, move_y));
        }
    }

    mod de {
        mod correct {
            use self::super::super::*;
            use self::super::super::rand::{Rng, thread_rng};
            use self::super::super::super::Message;
            use serde_json;

            #[test]
            fn welcome_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let speed = gen_f32(&mut rng);
                let size = gen_f32(&mut rng);
                let bullet_speed = gen_f32(&mut rng);
                let bullet_size = gen_f32(&mut rng);

                let expected_message = Message::Welcome {
                    id: id,
                    speed: speed,
                    size: size,
                    bullet_speed: bullet_speed,
                    bullet_size: bullet_size,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&welcome_expected_json(id, speed, size, bullet_speed, bullet_size))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn go_away_deserializes_properly() {
                let mut rng = thread_rng();
                let reason: String = {
                    let len = rng.gen_range(1, 100);
                    rng.gen_ascii_chars().take(len).collect()
                };

                let expected_message = Message::GoAway { reason: reason.clone() };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&go_away_expected_json(reason))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn player_joined_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();

                let expected_message = Message::PlayerJoined { id: id };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&player_joined_expected_json(id))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn player_left_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();

                let expected_message = Message::PlayerLeft { id: id };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&player_left_expected_json(id))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn shots_fired_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let bullet_id: u32 = rng.gen();
                let x = gen_f32(&mut rng);
                let y = gen_f32(&mut rng);
                let aim_x = gen_f32(&mut rng);
                let aim_y = gen_f32(&mut rng);

                let expected_message = Message::ShotsFired {
                    id: id,
                    bullet_id: bullet_id,
                    x: x,
                    y: y,
                    aim_x: aim_x,
                    aim_y: aim_y,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&shots_fired_expected_json(id, bullet_id, x, y, aim_x, aim_y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn player_spawned_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let x = gen_f32(&mut rng);
                let y = gen_f32(&mut rng);

                let expected_message = Message::PlayerSpawned {
                    id: id,
                    x: x,
                    y: y,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&player_spawned_expected_json(id, x, y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn player_destroyed_no_killer_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();

                let expected_message = Message::PlayerDestroyed {
                    id: id,
                    killer_id: None,
                    bullet_id: None,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&player_destroyed_no_killer_expected_json(id))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn player_destroyed_with_killer_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let killer_id: u32 = rng.gen();
                let bullet_id: u32 = rng.gen();

                let expected_message = Message::PlayerDestroyed {
                    id: id,
                    killer_id: Some(killer_id),
                    bullet_id: Some(bullet_id),
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&player_destroyed_with_killer_expected_json(id, killer_id, bullet_id))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn player_moving_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let x = gen_f32(&mut rng);
                let y = gen_f32(&mut rng);
                let move_x = gen_f32(&mut rng);
                let move_y = gen_f32(&mut rng);

                let expected_message = Message::PlayerMoving {
                    id: id,
                    x: x,
                    y: y,
                    move_x: move_x,
                    move_y: move_y,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&player_moving_expected_json(id, x, y, move_x, move_y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn player_stopped_deserializes_properly() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let x = gen_f32(&mut rng);
                let y = gen_f32(&mut rng);

                let expected_message = Message::PlayerStopped {
                    id: id,
                    x: x,
                    y: y,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&player_stopped_expected_json(id, x, y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn world_state_deserializes_properly() {
                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&world_state_expected_json())
                                                        .unwrap())
                               .unwrap(),
                           Message::WorldState);
            }

            #[test]
            fn start_moving_deserializes_properly() {
                let mut rng = thread_rng();
                let move_x = gen_f32(&mut rng);
                let move_y = gen_f32(&mut rng);

                let expected_message = Message::StartMoving {
                    move_x: move_x,
                    move_y: move_y,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&start_moving_expected_json(move_x, move_y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }

            #[test]
            fn stop_moving_deserializes_properly() {
                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&stop_moving_expected_json())
                                                        .unwrap())
                               .unwrap(),
                           Message::StopMoving);
            }

            #[test]
            fn fire_deserializes_properly() {
                let mut rng = thread_rng();
                let move_x = gen_f32(&mut rng);
                let move_y = gen_f32(&mut rng);

                let expected_message = Message::Fire {
                    move_x: move_x,
                    move_y: move_y,
                };

                assert_eq!(str::parse::<Message>(&*&serde_json::to_string(&fire_expected_json(move_x, move_y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
            }
        }

        mod incorrect {
            use std::collections::BTreeMap;
            use self::super::super::*;
            use self::super::super::rand::{Rng, thread_rng};
            use self::super::super::super::{Message, MessageError};
            use serde_json;

            #[test]
            fn player_destroyed_with_killer_no_bullet_fails() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let killer_id: u32 = rng.gen();

                let mut unexpected_json = player_destroyed_with_killer_expected_json(id,
                                                                                     killer_id,
                                                                                     0);
                let _ = unexpected_json.as_object_mut()
                                       .unwrap()
                                       .get_mut("data")
                                       .unwrap()
                                       .as_object_mut()
                                       .unwrap()
                                       .remove("bullet_id")
                                       .unwrap();

                match str::parse::<Message>(&*&serde_json::to_string(&unexpected_json).unwrap())
                          .unwrap_err() {
                    MessageError::PropertyMissing(_) => {}
                    _ => panic!("Incorrect error kind"),
                }
            }

            #[test]
            fn player_destroyed_with_bullet_no_killer_fails() {
                let mut rng = thread_rng();
                let id: u32 = rng.gen();
                let bullet_id: u32 = rng.gen();

                let mut unexpected_json = player_destroyed_with_killer_expected_json(id,
                                                                                     bullet_id,
                                                                                     0);
                let _ = unexpected_json.as_object_mut()
                                       .unwrap()
                                       .get_mut("data")
                                       .unwrap()
                                       .as_object_mut()
                                       .unwrap()
                                       .remove("killer_id")
                                       .unwrap();

                match str::parse::<Message>(&*&serde_json::to_string(&unexpected_json).unwrap())
                          .unwrap_err() {
                    MessageError::PropertyMissing(_) => {}
                    me => {
                        panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me))
                    }
                }
            }

            #[test]
            fn missing_type_fails() {
                let mut unexpected_json = player_joined_expected_json(0);
                let _ = unexpected_json.as_object_mut()
                                       .unwrap()
                                       .remove("type")
                                       .unwrap();

                match str::parse::<Message>(&*&serde_json::to_string(&unexpected_json).unwrap())
                          .unwrap_err() {
                    MessageError::PropertyMissing(_) => {}
                    me => {
                        panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me))
                    }
                }
            }

            #[test]
            fn missing_data_fails() {
                let mut unexpected_json = player_joined_expected_json(0);
                let _ = unexpected_json.as_object_mut()
                                       .unwrap()
                                       .remove("data")
                                       .unwrap();

                match str::parse::<Message>(&*&serde_json::to_string(&unexpected_json).unwrap())
                          .unwrap_err() {
                    MessageError::PropertyMissing(_) => {}
                    me => {
                        panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me))
                    }
                }
            }

            #[test]
            fn missing_data_subkey_fails() {
                let mut unexpected_json = player_joined_expected_json(0);
                let _ = unexpected_json.as_object_mut()
                                       .unwrap()
                                       .get_mut("data")
                                       .unwrap()
                                       .as_object_mut()
                                       .unwrap()
                                       .clear();

                match str::parse::<Message>(&*&serde_json::to_string(&unexpected_json).unwrap())
                          .unwrap_err() {
                    MessageError::PropertyMissing(_) => {}
                    me => {
                        panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me))
                    }
                }
            }

            #[test]
            fn empty_toplevel_object_fails() {
                let unexpected_json = serde_json::Value::Object(BTreeMap::new());

                match str::parse::<Message>(&*&serde_json::to_string(&unexpected_json).unwrap())
                          .unwrap_err() {
                    MessageError::PropertyMissing(_) => {}
                    me => {
                        panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me))
                    }
                }
            }

            #[test]
            fn incorrect_toplevel_type_fails() {
                let unexpected_json = serde_json::Value::Null;

                match str::parse::<Message>(&*&serde_json::to_string(&unexpected_json).unwrap())
                          .unwrap_err() {
                    MessageError::BadType(_) => {}
                    me => panic!(format!("Incorrect error kind: {:?}, should be BadType", me)),
                }
            }
        }
    }


    pub fn welcome_expected_json(id: u32,
                                 speed: f32,
                                 size: f32,
                                 bullet_speed: f32,
                                 bullet_size: f32)
                                 -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("welcome".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("speed".to_string(), Value::F64(speed as f64)),
                    ("size".to_string(), Value::F64(size as f64)),
                    ("bullet_speed".to_string(), Value::F64(bullet_speed as f64)),
                    ("bullet_size".to_string(), Value::F64(bullet_size as f64)),
                ]
            ))),
        ]))
    }

    pub fn go_away_expected_json(reason: String) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("go_away".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("reason".to_string(), Value::String(reason)),
                ]
            ))),
        ]))
    }

    pub fn player_joined_expected_json(id: u32) -> Value {
        id_only_expected_json(id, "player_joined")
    }

    pub fn player_left_expected_json(id: u32) -> Value {
        id_only_expected_json(id, "player_left")
    }

    pub fn shots_fired_expected_json(id: u32,
                                     bullet_id: u32,
                                     x: f32,
                                     y: f32,
                                     aim_x: f32,
                                     aim_y: f32)
                                     -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("shots_fired".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("bullet_id".to_string(), Value::U64(bullet_id as u64)),
                    ("x".to_string(), Value::F64(x as f64)),
                    ("y".to_string(), Value::F64(y as f64)),
                    ("aim_x".to_string(), Value::F64(aim_x as f64)),
                    ("aim_y".to_string(), Value::F64(aim_y as f64)),
                ]
            ))),
        ]))
    }

    pub fn player_spawned_expected_json(id: u32, x: f32, y: f32) -> Value {
        id_pos_expected_json(id, x, y, "player_spawned")
    }

    pub fn player_destroyed_no_killer_expected_json(id: u32) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_destroyed".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                ]
            ))),
        ]))
    }

    pub fn player_destroyed_with_killer_expected_json(id: u32,
                                                      killer_id: u32,
                                                      bullet_id: u32)
                                                      -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("player_destroyed".to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("killer_id".to_string(), Value::U64(killer_id as u64)),
                    ("bullet_id".to_string(), Value::U64(bullet_id as u64)),
                ]
            ))),
        ]))
    }

    pub fn player_moving_expected_json(id: u32, x: f32, y: f32, move_x: f32, move_y: f32) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
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
        ]))
    }

    pub fn player_stopped_expected_json(id: u32, x: f32, y: f32) -> Value {
        id_pos_expected_json(id, x, y, "player_stopped")
    }

    pub fn world_state_expected_json() -> Value {
        // TODO implement world_state
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("world_state".to_string())),
        ]))
    }

    pub fn start_moving_expected_json(move_x: f32, move_y: f32) -> Value {
        movement_expected_json(move_x, move_y, "start_moving")
    }

    pub fn stop_moving_expected_json() -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String("stop_moving".to_string())),
        ]))
    }

    pub fn fire_expected_json(move_x: f32, move_y: f32) -> Value {
        movement_expected_json(move_x, move_y, "fire")
    }

    fn id_only_expected_json(id: u32, msg_type: &str) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String(msg_type.to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                ]
            ))),
        ]))
    }

    fn id_pos_expected_json(id: u32, x: f32, y: f32, msg_type: &str) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String(msg_type.to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("id".to_string(), Value::U64(id as u64)),
                    ("x".to_string(), Value::F64(x as f64)),
                    ("y".to_string(), Value::F64(y as f64)),
                ]
            ))),
        ]))
    }

    fn movement_expected_json(move_x: f32, move_y: f32, msg_type: &str) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("type".to_string(), Value::String(msg_type.to_string())),
            ("data".to_string(), Value::Object(
                BTreeMap::from_iter(vec![
                    ("move_x".to_string(), Value::F64(move_x as f64)),
                    ("move_y".to_string(), Value::F64(move_y as f64)),
                ]
            ))),
        ]))
    }


    pub fn gen_f32<R: Rng>(rng: &mut R) -> f32 {
        // Randoming actual floats hits us when widening them to f64
        (rng.gen_range(0u32, 99u32) as f32) + 0.5f32
    }
}
