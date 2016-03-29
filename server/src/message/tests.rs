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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
                   go_away_expected_json(reason));
    }

    #[test]
    fn player_joined_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();

        let json_txt = Message::PlayerJoined { id: id }.to_string();

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
                   player_joined_expected_json(id));
    }

    #[test]
    fn player_left_serializes_properly() {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();

        let json_txt = Message::PlayerLeft { id: id }.to_string();

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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


        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
                   player_stopped_expected_json(id, x, y));
    }

    #[test]
    fn world_state_serializes_properly() {
        // TODO implement WorldState
        assert_eq!(serde_json::from_str::<Value>(&Message::WorldState.to_string()).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
                   start_moving_expected_json(move_x, move_y));
    }

    #[test]
    fn stop_moving_serializes_properly() {
        let json_txt = Message::StopMoving.to_string();

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

        assert_eq!(serde_json::from_str::<Value>(&json_txt).unwrap(),
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&welcome_expected_json(id, speed, size, bullet_speed, bullet_size))
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&go_away_expected_json(reason))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
        }

        #[test]
        fn player_joined_deserializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();

            let expected_message = Message::PlayerJoined { id: id };

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&player_joined_expected_json(id))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
        }

        #[test]
        fn player_left_deserializes_properly() {
            let mut rng = thread_rng();
            let id: u32 = rng.gen();

            let expected_message = Message::PlayerLeft { id: id };

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&player_left_expected_json(id))
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&shots_fired_expected_json(id, bullet_id, x, y, aim_x, aim_y))
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&player_spawned_expected_json(id, x, y))
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&player_destroyed_no_killer_expected_json(id))
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&player_destroyed_with_killer_expected_json(id, killer_id, bullet_id))
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&player_moving_expected_json(id, x, y, move_x, move_y))
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&player_stopped_expected_json(id, x, y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
        }

        #[test]
        fn world_state_deserializes_properly() {
            assert_eq!(str::parse::<Message>(&serde_json::to_string(&world_state_expected_json())
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&start_moving_expected_json(move_x, move_y))
                                                        .unwrap())
                               .unwrap(),
                           expected_message);
        }

        #[test]
        fn stop_moving_deserializes_properly() {
            assert_eq!(str::parse::<Message>(&serde_json::to_string(&stop_moving_expected_json())
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

            assert_eq!(str::parse::<Message>(&serde_json::to_string(&fire_expected_json(move_x,
                                                                                        move_y))
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

            let mut unexpected_json = player_destroyed_with_killer_expected_json(id, killer_id, 0);
            let _ = unexpected_json.as_object_mut()
                                   .unwrap()
                                   .get_mut("data")
                                   .unwrap()
                                   .as_object_mut()
                                   .unwrap()
                                   .remove("bullet_id")
                                   .unwrap();

            match str::parse::<Message>(&serde_json::to_string(&unexpected_json).unwrap())
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

            let mut unexpected_json = player_destroyed_with_killer_expected_json(id, bullet_id, 0);
            let _ = unexpected_json.as_object_mut()
                                   .unwrap()
                                   .get_mut("data")
                                   .unwrap()
                                   .as_object_mut()
                                   .unwrap()
                                   .remove("killer_id")
                                   .unwrap();

            match str::parse::<Message>(&serde_json::to_string(&unexpected_json).unwrap())
                      .unwrap_err() {
                MessageError::PropertyMissing(_) => {}
                me => panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me)),
            }
        }

        #[test]
        fn missing_type_fails() {
            let mut unexpected_json = player_joined_expected_json(0);
            let _ = unexpected_json.as_object_mut()
                                   .unwrap()
                                   .remove("type")
                                   .unwrap();

            match str::parse::<Message>(&serde_json::to_string(&unexpected_json).unwrap())
                      .unwrap_err() {
                MessageError::PropertyMissing(_) => {}
                me => panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me)),
            }
        }

        #[test]
        fn missing_data_fails() {
            let mut unexpected_json = player_joined_expected_json(0);
            let _ = unexpected_json.as_object_mut()
                                   .unwrap()
                                   .remove("data")
                                   .unwrap();

            match str::parse::<Message>(&serde_json::to_string(&unexpected_json).unwrap())
                      .unwrap_err() {
                MessageError::PropertyMissing(_) => {}
                me => panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me)),
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

            match str::parse::<Message>(&serde_json::to_string(&unexpected_json).unwrap())
                      .unwrap_err() {
                MessageError::PropertyMissing(_) => {}
                me => panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me)),
            }
        }

        #[test]
        fn empty_toplevel_object_fails() {
            let unexpected_json = serde_json::Value::Object(BTreeMap::new());

            match str::parse::<Message>(&serde_json::to_string(&unexpected_json).unwrap())
                      .unwrap_err() {
                MessageError::PropertyMissing(_) => {}
                me => panic!(format!("Incorrect error kind: {:?}, should be PropertyMissing", me)),
            }
        }

        #[test]
        fn incorrect_toplevel_type_fails() {
            let unexpected_json = serde_json::Value::Null;

            match str::parse::<Message>(&serde_json::to_string(&unexpected_json).unwrap())
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
