use tari_comms::message::MessageExt;
use tari_comms::Bytes;

#[derive(Debug, Clone)]
pub struct Message {
    ty: MessageType,
    payload: Bytes,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    NewGame,
    Move,
    Resign,
}

impl Message {
    pub fn new<T: MessageExt>(ty: MessageType, payload: T) -> Self {
        Self {
            ty,
            payload: payload.to_encoded_bytes().into(),
        }
    }
}
