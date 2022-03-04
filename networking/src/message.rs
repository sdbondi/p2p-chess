use anyhow::anyhow;

#[derive(Clone, prost::Message)]
pub struct ProtoMessage {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(uint32, tag = "2")]
    pub seq: u32,
    #[prost(enumeration = "MessageType", tag = "3")]
    pub message_type: i32,
    #[prost(bytes, tag = "4")]
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, prost::Enumeration)]
pub enum MessageType {
    NewGame = 0,
    PlayMove = 1,
    Resign = 2,
}

impl ProtoMessage {
    pub fn new<T: prost::Message>(id: u32, seq: u32, message_type: MessageType, payload: T) -> Self {
        let mut bytes = Vec::with_capacity(payload.encoded_len());
        payload.encode(&mut bytes).unwrap();
        Self {
            id,
            seq,
            message_type: message_type as i32,
            payload: bytes,
        }
    }
}

impl TryFrom<i32> for MessageType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::NewGame),
            1 => Ok(MessageType::PlayMove),
            2 => Ok(MessageType::Resign),
            _ => Err(anyhow!("Invalid message type {}", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message<T> {
    pub id: u32,
    pub seq: u32,
    pub message_type: MessageType,
    pub payload: T,
}

impl<T: prost::Message> Message<T> {
    pub fn new(id: u32, seq: u32, message_type: MessageType, payload: T) -> Self {
        Self {
            id,
            seq,
            message_type,
            payload,
        }
    }

    pub fn to_proto_message(&self) -> ProtoMessage {
        let mut bytes = Vec::with_capacity(self.payload.encoded_len());
        self.payload.encode(&mut bytes).unwrap();
        ProtoMessage::new(self.id, self.seq, self.message_type, bytes)
    }
}

impl<T: prost::Message + Default> TryFrom<ProtoMessage> for Message<T> {
    type Error = anyhow::Error;

    fn try_from(value: ProtoMessage) -> Result<Self, Self::Error> {
        let payload = T::decode(value.payload.as_slice())?;
        Ok(Message {
            id: value.id,
            seq: value.seq,
            message_type: MessageType::try_from(value.message_type)?,
            payload,
        })
    }
}

#[derive(Clone, prost::Message)]
pub struct NewGameMsg {
    #[prost(uint32, tag = "1")]
    pub player: u32,
}

#[derive(Clone, prost::Message)]
pub struct MoveMsg {
    #[prost(uint32, tag = "1")]
    pub mv: u32,
    #[prost(string, tag = "2")]
    pub board: String,
}

#[derive(Clone, prost::Message)]
pub struct ResignMsg;
