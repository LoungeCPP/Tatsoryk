//! The common data formats for cross-thread events.
//!
//! The game itself consists of multiple threads, a single game loop thread as well as multiple server threads.
//! These threads communicate back and forth between each other using a couple of mpsc channels.
//! This file defines the common data formats for those channels.

use std::sync::mpsc::{Sender, SendError};
use std::fmt;

use message;

/// This represents a single websocket connected to the game.
#[derive(Clone)]
pub struct Client {
    /// The unique id for the client.
    pub id: u32,

    /// 'sender' is a channel which allows you to send messages to the corresponding websocket.
    ///
    /// Send a None to close the websocket. (Some(data) for a normal message).
    sender: Sender<Option<String>>,
}

impl Client {
    /// Create a new client from a given id and sender channel.
    pub fn new(id: u32, sender: Sender<Option<String>>) -> Client {
        Client {
            id: id,
            sender: sender,
        }
    }

    /// Send a message to the websocket.
    pub fn send(&self, message: String) -> Result<(), SendError<Option<String>>> {
        self.sender.send(Some(message))
    }

    /// Close the websocket.
    pub fn close(&self) -> Result<(), SendError<Option<String>>> {
        self.sender.send(None)
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client {}", self.id)
    }
}

/// A WebSocketEvent is any websocket message which might be sent to the main game loop.
///
/// Right now, we have clients connecting, disconnecting, and sending messages.
/// This is the place where we would add additional stuff like say, unix signals.
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    ClientCreated {
        client: Client,
    },
    ClientClosed {
        client_id: u32,
    },
    ClientMessage {
        client_id: u32,
        message: message::Message,
    },
}
