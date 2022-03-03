use tokio::sync::mpsc;
pub use tokio::sync::mpsc::error::{SendError, TryRecvError, TrySendError};

pub fn channel<T>(capacity: usize) -> (MessageChannel<T>, MessageChannel<T>) {
    let (ltr_tx, ltr_rx) = mpsc::channel(capacity);
    let (rtl_tx, rtl_rx) = mpsc::channel(capacity);
    (
        MessageChannel {
            sender: ltr_tx,
            receiver: rtl_rx,
        },
        MessageChannel {
            sender: rtl_tx,
            receiver: ltr_rx,
        },
    )
}

#[derive(Debug)]
pub struct MessageChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

impl<T> MessageChannel<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }

    pub async fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.sender.send(value).await
    }

    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        self.sender.try_send(value)
    }
}
