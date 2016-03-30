use serde_json;

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
