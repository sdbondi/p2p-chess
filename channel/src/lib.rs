mod channel;

pub use channel::{channel, MessageChannel, SendError, TryRecvError};
use tari_comms::types::CommsPublicKey;

#[derive(Debug, Clone)]
pub struct ChessOperation {
    pub game_id: u32,
    pub seq: u32,
    pub to: CommsPublicKey,
    pub from: CommsPublicKey,
    pub operation: OperationType,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    NewGame { player: u8 },
    MovePlayed { mv: u16, board: String },
    Resign,
}
