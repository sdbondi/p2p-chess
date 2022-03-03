mod handle;
mod message;
mod node;

use std::{
    fs,
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::Arc,
};

use anyhow::anyhow;
use p2p_chess_channel::{ChessOperation, MessageChannel, OperationType};
// Re-exports
pub use tari_comms::{
    multiaddr::Multiaddr,
    peer_manager::{NodeIdentity, PeerFeatures},
};
use tari_comms::{types::CommsPublicKey, CommsNode};
use tari_comms_dht::{
    domain_message::OutboundDomainMessage,
    inbound::DecryptedDhtMessage,
    outbound::OutboundEncryption,
    Dht,
};
use tari_shutdown::ShutdownSignal;
use tokio::{sync::mpsc, task};

use crate::message::{Message, MessageType, MoveMsg, NewGameMsg, ProtoMessage, ResignMsg};

pub struct Networking {
    node: CommsNode,
    dht: Dht,
    in_msg: mpsc::Receiver<DecryptedDhtMessage>,

    channel: MessageChannel<ChessOperation>,
    node_identity: Arc<NodeIdentity>,
}

impl Networking {
    pub async fn start<P: AsRef<Path>>(
        node_identity: Arc<NodeIdentity>,
        base_path: P,
        channel: MessageChannel<ChessOperation>,
        shutdown_signal: ShutdownSignal,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(base_path.as_ref())?;
        let tor_identity = load_json(base_path.as_ref().join("tor.json"))?;
        let seed_peers = vec![];
        let (node, dht, in_msg) = node::create(
            node_identity.clone(),
            base_path.as_ref().join("db"),
            tor_identity,
            9999,
            seed_peers,
            shutdown_signal,
        )
        .await?;

        let worker = Self {
            node,
            dht,
            in_msg,
            channel,
            node_identity: node_identity.clone(),
        };
        worker.spawn();

        Ok(())
    }

    fn spawn(self) {
        task::spawn(self.run_event_loop());
    }

    async fn run_event_loop(mut self) {
        loop {
            let res = tokio::select! {
                Some(msg) = self.channel.recv() => self.handle_operation(msg).await,
                Some(msg) = self.in_msg.recv() => self.handle_inbound_message(msg).await,
            };
            if let Err(err) = res {
                log::error!("{}", err);
            }
        }
    }

    async fn handle_operation(&self, op: ChessOperation) -> anyhow::Result<()> {
        match op.operation {
            OperationType::NewGame { player } => {
                self.broadcast_msg(
                    op.opponent,
                    Message::new(op.seq, MessageType::NewGame, NewGameMsg { player: player as u32 }),
                )
                .await?;
            },
            OperationType::MovePlayed(mv) => {
                self.broadcast_msg(
                    op.opponent,
                    Message::new(op.seq, MessageType::PlayMove, MoveMsg { value: mv as u32 }),
                )
                .await?;
            },
            OperationType::Resign => {
                self.broadcast_msg(op.opponent, Message::new(op.seq, MessageType::Resign, ResignMsg))
                    .await?;
            },
        }

        Ok(())
    }

    async fn handle_inbound_message(&self, msg: DecryptedDhtMessage) -> anyhow::Result<()> {
        let public_key = msg.source_peer.public_key.clone();
        match msg.success() {
            Some(body) => {
                let msg = body.decode_part::<ProtoMessage>(0)?.ok_or_else(|| anyhow!("No msg"))?;
                let msg_type = msg.message_type.try_into()?;
                let op = match msg_type {
                    MessageType::NewGame => {
                        let msg = Message::<NewGameMsg>::try_from(msg)?;
                        ChessOperation {
                            seq: msg.seq,
                            opponent: public_key,
                            operation: OperationType::NewGame {
                                player: msg.payload.player as u8,
                            },
                        }
                    },
                    MessageType::PlayMove => {
                        let msg = Message::<MoveMsg>::try_from(msg)?;
                        ChessOperation {
                            seq: msg.seq,
                            opponent: public_key,
                            operation: OperationType::MovePlayed(msg.payload.value as u16),
                        }
                    },
                    MessageType::Resign => {
                        let msg = Message::<ResignMsg>::try_from(msg)?;
                        ChessOperation {
                            seq: msg.seq,
                            opponent: public_key,
                            operation: OperationType::Resign,
                        }
                    },
                };

                self.channel.send(op).await?;
            },
            None => {
                log::warn!("🤷‍ Received message we could not decrypt {:?}", msg);
            },
        }

        Ok(())
    }

    async fn broadcast_msg<T: prost::Message>(
        &self,
        public_key: CommsPublicKey,
        msg: Message<T>,
    ) -> anyhow::Result<()> {
        let msg = msg.to_proto_message();
        self.dht
            .outbound_requester()
            .propagate(
                public_key.clone().into(),
                OutboundEncryption::EncryptFor(Box::new(public_key)),
                vec![],
                OutboundDomainMessage::new(999, msg),
            )
            .await?;

        Ok(())
    }
}

fn load_json<T: serde::de::DeserializeOwned, P: AsRef<Path>>(path: P) -> anyhow::Result<Option<T>> {
    if !path.as_ref().exists() {
        return Ok(None);
    }

    let mut buf = Vec::new();
    File::open(path)?.read_to_end(&mut buf)?;
    let t = serde_json::from_slice(&buf)?;
    Ok(Some(t))
}

fn save_json<T: serde::Serialize, P: AsRef<Path>>(path: P, item: &T) -> anyhow::Result<()> {
    fs::create_dir_all(&path)?;
    let buf = serde_json::to_vec(item)?;
    File::create(path)?.write_all(&buf)?;
    Ok(())
}
