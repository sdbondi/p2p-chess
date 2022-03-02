use prost::Message as ProstMessage;
use std::ops::Deref;

#[derive(Clone, prost::Message)]
pub struct ProtoMessage {
    #[prost(uint32, tag = "1")]
    seq: u32,
    #[prost(enumeration = "MessageType", tag = "2")]
    message_type: i32,
    #[prost(bytes, tag = "3")]
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, prost::Enumeration)]
pub enum MessageType {
    NewGame = 0,
    Move = 1,
    Resign = 2,
}

impl ProtoMessage {
    pub fn new<T: prost::Message>(seq: u32, message_type: MessageType, payload: T) -> Self {
        let mut bytes = Vec::with_capacity(payload.encoded_len());
        payload.encode(&mut bytes).unwrap();
        Self {
            seq,
            message_type: message_type as i32,
            payload: bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message<T> {
    seq: u32,
    message_type: MessageType,
    payload: T,
}

impl<T: prost::Message> Message<T> {
    pub fn new(seq: u32, message_type: MessageType, payload: T) -> Self {
        Self {
            seq,
            message_type,
            payload,
        }
    }

    pub fn to_proto_message(&self) -> ProtoMessage {
        let mut bytes = Vec::with_capacity(self.payload.encoded_len());
        self.payload.encode(&mut bytes).unwrap();
        ProtoMessage::new(self.seq, self.message_type, bytes)
    }
}
