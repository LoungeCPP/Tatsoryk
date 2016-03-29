use std::str::FromStr;
use std::collections::BTreeMap;
use self::super::MessageError;
use serde_json;

macro_rules! player_or_bullet {
    ($name:ident, $name_s:expr) => {
/// Part of the **world_state** message, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec).")]
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $name {
            id: u32,
            x: f32,
            y: f32,
            move_x: Option<f32>,
            move_y: Option<f32>,
        }

        impl $name {
// `static` would work better but it's a keyword so it doesn't
            pub fn not_moving(id: u32, x: f32, y: f32) -> Self {
                $name {
                    id: id,
                    x: x,
                    y: y,
                    move_x: None,
                    move_y: None,
                }
            }

            pub fn moving(id: u32, x: f32, y: f32, move_x: f32, move_y: f32) -> Self {
                $name {
                    id: id,
                    x: x,
                    y: y,
                    move_x: Some(move_x),
                    move_y: Some(move_y),
                }
            }

            pub fn to_json(&self) -> serde_json::Value {
                let mut values = BTreeMap::new();
                let _ = values.insert("id".to_string(), serde_json::Value::U64(self.id as u64));
                let _ = values.insert("x".to_string(), serde_json::Value::F64(self.x as f64));
                let _ = values.insert("y".to_string(), serde_json::Value::F64(self.y as f64));

                match (self.move_x, self.move_y) {
                    (Some(move_x), Some(move_y)) => {
                        let _ = values.insert("move_x".to_string(), serde_json::Value::F64(move_x as f64));
                        let _ = values.insert("move_y".to_string(), serde_json::Value::F64(move_y as f64));
                    }
                    (None, None) => {}
                    _ => panic!("move_x and move_y must be either both Some or both None"),
                }

                serde_json::Value::Object(values)
            }

            pub fn from_json(json: &serde_json::Value) -> Result<Self, MessageError> {
                match json.as_object() {
                    Some(msg) => {
                        let keys = msg.keys().collect::<Vec<_>>();
                        if keys != vec!["id", "move_x", "move_y", "x", "y"] &&
                           keys != vec!["id", "x", "y"] {
                            return Err(MessageError::PropertyMissing(
                                format!(concat!($name_s, r#" Object is a mismatch for `"{{"id", "x", "y"[, "move_x", "move_y"]}}"`: {:?}"#), keys)));
                        }

                        let id = try!(unpack_u32(msg.get("id").unwrap()));
                        let x = try!(unpack_f32(msg.get("x").unwrap()));
                        let y = try!(unpack_f32(msg.get("y").unwrap()));
                        let move_x = match msg.get("move_x") {
                            Some(move_x) => Some(try!(unpack_f32(move_x))),
                            None => None,
                        };
                        let move_y = match msg.get("move_y") {
                            Some(move_y) => Some(try!(unpack_f32(move_y))),
                            None => None,
                        };

                        Ok($name{
                            id: id,
                            x: x,
                            y: y,
                            move_x: move_x,
                            move_y: move_y,
                        })
                    }
                    None => Err(MessageError::BadType(concat!($name_s, " JSON not an Object").to_string())),
                }
            }
        }
    }
}

player_or_bullet!(Player, "Player");
player_or_bullet!(Bullet, "Bullet");

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

/// Only testing Player, because Bullet is literally identical
#[cfg(test)]
mod tests {
    extern crate rand;

    use std::iter::FromIterator;
    use std::collections::BTreeMap;
    use self::rand::{thread_rng, Rng};
    use serde_json::{self, Value};
    use self::super::Player;
    use self::super::super::MessageError;

    #[test]
    fn static_player_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);

        assert_eq!(Player::not_moving(id, x, y).to_json(),
                   static_player_expected_json(id, x, y));
    }

    #[test]
    fn moving_player_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);
        let move_x = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        assert_eq!(Player::moving(id, x, y, move_x, move_y).to_json(),
                   moving_player_expected_json(id, x, y, move_x, move_y));
    }

    #[test]
    fn static_player_deserializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);

        assert_eq!(Player::from_json(&static_player_expected_json(id, x, y)).unwrap(),
                   Player::not_moving(id, x, y));
    }

    #[test]
    fn moving_player_deserializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);
        let move_x = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        assert_eq!(Player::from_json(&moving_player_expected_json(id, x, y, move_x, move_y)).unwrap(),
                   Player::moving(id, x, y, move_x, move_y));
    }

    #[test]
    fn player_with_move_x_no_move_y_deserialize_fails() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);
        let move_x = gen_f32(&mut rng);

        let mut unexpected_json = moving_player_expected_json(id, x, y, move_x, 0f32);
        let _ = unexpected_json.as_object_mut()
                               .unwrap()
                               .remove("move_y")
                               .unwrap();

        match Player::from_json(&unexpected_json).unwrap_err() {
            MessageError::PropertyMissing(_) => {}
            me => panic!(format!("Incorrect error type: {:?}, should be PropertyMissing", me)),
        }
    }

    #[test]
    fn player_with_move_y_no_move_x_deserialize_fails() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let x = gen_f32(&mut rng);
        let y = gen_f32(&mut rng);
        let move_y = gen_f32(&mut rng);

        let mut unexpected_json = moving_player_expected_json(id, x, y, 0f32, move_y);
        let _ = unexpected_json.as_object_mut()
                               .unwrap()
                               .remove("move_x")
                               .unwrap();

        match Player::from_json(&unexpected_json).unwrap_err() {
            MessageError::PropertyMissing(_) => {}
            me => panic!(format!("Incorrect error type: {:?}, should be PropertyMissing", me)),
        }
    }

    fn static_player_expected_json(id: u32, x: f32, y: f32) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("id".to_string(), Value::U64(id as u64)),
            ("x".to_string(), Value::F64(x as f64)),
            ("y".to_string(), Value::F64(y as f64)),
        ]))
    }

    fn moving_player_expected_json(id: u32, x: f32, y: f32, move_x: f32, move_y: f32) -> Value {
        Value::Object(BTreeMap::from_iter(vec![
            ("id".to_string(), Value::U64(id as u64)),
            ("x".to_string(), Value::F64(x as f64)),
            ("y".to_string(), Value::F64(y as f64)),
            ("move_x".to_string(), Value::F64(move_x as f64)),
            ("move_y".to_string(), Value::F64(move_y as f64)),
        ]))
    }

    fn gen_f32<R: Rng>(rng: &mut R) -> f32 {
        // Randoming actual floats hits us when widening them to f64
        (rng.gen_range(0u32, 99u32) as f32) + 0.5f32
    }
}
