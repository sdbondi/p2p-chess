use crate::message::{MessageType, ProtoMessage};
use std::sync::Arc;
use tari_comms::types::CommsPublicKey;
use tari_comms::NodeIdentity;
use tokio::sync::mpsc;

pub struct NetworkingHandle {
    // TODO: remove "I'm lazy" pubs
    pub(crate) inbound_messages: mpsc::Receiver<ProtoMessage>,
    pub(crate) outbound_messages: mpsc::Sender<ProtoMessage>,
    pub(crate) node_identity: Arc<NodeIdentity>,
}

impl NetworkingHandle {
    pub async fn send_message<T: prost::Message>(
        &self,
        seq: u32,
        message_type: MessageType,
        msg: T,
    ) -> anyhow::Result<()> {
        self.outbound_messages
            .send(ProtoMessage::new(seq, message_type, msg))
            .await?;
        Ok(())
    }

    pub async fn next_msg(&self) -> Option<ProtoMessage> {
        self.inbound_messages.recv().await
    }

    pub fn public_key(&self) -> &CommsPublicKey {
        self.node_identity.public_key()
    }
}
