use tari_common_types::types::PublicKey;
use tokio::sync::{broadcast, mpsc};

pub type CommandSubscription = broadcast::Receiver<ChessOperation>;

#[derive(Debug, Clone)]
pub struct CommandPublisher(broadcast::Sender<ChessOperation>);

pub type CommandReceiver = mpsc::Receiver<ChessOperation>;

impl CommandPublisher {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(10);
        Self(tx)
    }

    pub fn subscribe(&self) -> CommandSubscription {
        self.0.subscribe()
    }

    pub fn publish(&self, cmd: ChessOperation) -> anyhow::Result<()> {
        self.0.send(cmd)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChessOperation {
    pub opponent: PublicKey,
    pub operation: OperationType,
}

#[derive(Debug, Clone, Copy)]
pub enum OperationType {
    NewGame { player: u8 },
    MovePlayed(u16),
    Resign,
}
