mod channel;

pub use channel::{channel, MessageChannel, SendError, TryRecvError};
use tari_comms::types::CommsPublicKey;

#[derive(Debug, Clone)]
pub struct ChessOperation {
    pub seq: u32,
    pub opponent: CommsPublicKey,
    pub operation: OperationType,
}

#[derive(Debug, Clone, Copy)]
pub enum OperationType {
    NewGame { player: u8 },
    MovePlayed(u16),
    Resign,
}
