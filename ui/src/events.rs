use tokio::sync::broadcast;

pub type EventSubscription = broadcast::Receiver<ChessUiEvent>;

#[derive(Debug, Clone)]
pub struct EventPublisher(broadcast::Sender<ChessUiEvent>);

impl EventPublisher {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(10);
        Self(tx)
    }

    pub fn subscribe(&self) -> EventSubscription {
        self.0.subscribe()
    }

    pub fn publish(&self, event: ChessUiEvent) -> anyhow::Result<()> {
        self.0.send(event)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ChessUiEvent {
    NewGame { public_key: String },
    GameEnded,
}
