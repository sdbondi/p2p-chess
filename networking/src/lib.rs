mod message;
mod node;

use std::{
    fs,
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use p2p_chess_channel::{ChessOperation, MessageChannel, OperationType};
use rand::{rngs::OsRng, Rng};
// Re-exports
pub use tari_comms::{
    multiaddr::Multiaddr,
    peer_manager::{NodeIdentity, PeerFeatures},
};
use tari_comms::{
    peer_manager::{NodeId, Peer},
    types::CommsPublicKey,
};
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
        // TODO
        let seed_peers = vec![
            peer_from_str(
                "c2eca9cf32261a1343e21ed718e79f25bfc74386e9305350b06f62047f519347::/onion3/6yxqk2ybo43u73ukfhyc42qn25echn4zegjpod2ccxzr2jd5atipwzqd:18141",
            )
            .unwrap(),
        ];
        let port = OsRng.gen_range(15000..50000);
        let (node, dht, in_msg) = node::create(
            node_identity.clone(),
            base_path.as_ref().join("db"),
            tor_identity,
            port,
            seed_peers,
            shutdown_signal,
        )
        .await?;
        save_json(base_path.as_ref().join("node-identity.json"), node.node_identity_ref())?;

        node.connectivity()
            .wait_for_connectivity(Duration::from_secs(30))
            .await?;

        let worker = Self {
            dht,
            in_msg,
            channel,
            node_identity: node.node_identity(),
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
        dbg!(&op);
        match op.operation {
            OperationType::NewGame { player } => {
                self.broadcast_msg(
                    op.to,
                    Message::new(op.game_id, op.seq, MessageType::NewGame, NewGameMsg {
                        player: player as u32,
                    }),
                )
                .await?;
            },
            OperationType::MovePlayed { board, mv } => {
                self.broadcast_msg(
                    op.to,
                    Message::new(op.game_id, op.seq, MessageType::PlayMove, MoveMsg {
                        mv: mv as u32,
                        board,
                    }),
                )
                .await?;
            },
            OperationType::Resign => {
                self.broadcast_msg(op.to, Message::new(op.game_id, op.seq, MessageType::Resign, ResignMsg))
                    .await?;
            },
        }

        Ok(())
    }

    async fn handle_inbound_message(&self, msg: DecryptedDhtMessage) -> anyhow::Result<()> {
        let src_public_key = msg
            .authenticated_origin
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Message origin not authenticated. Ignoring message."))?;
        match msg.success() {
            Some(body) => {
                let msg = body.decode_part::<ProtoMessage>(1)?.ok_or_else(|| anyhow!("No msg"))?;
                let msg_type = msg.message_type.try_into()?;
                let op = match msg_type {
                    MessageType::NewGame => {
                        let msg = Message::<NewGameMsg>::try_from(msg)?;
                        ChessOperation {
                            game_id: msg.id,
                            seq: msg.seq,
                            to: self.node_identity.public_key().clone(),
                            from: src_public_key,
                            operation: OperationType::NewGame {
                                player: msg.payload.player as u8,
                            },
                        }
                    },
                    MessageType::PlayMove => {
                        let msg = Message::<MoveMsg>::try_from(msg)?;
                        ChessOperation {
                            game_id: msg.id,
                            seq: msg.seq,
                            to: self.node_identity.public_key().clone(),
                            from: src_public_key,
                            operation: OperationType::MovePlayed {
                                mv: msg.payload.mv as u16,
                                board: msg.payload.board,
                            },
                        }
                    },
                    MessageType::Resign => {
                        let msg = Message::<ResignMsg>::try_from(msg)?;
                        ChessOperation {
                            game_id: msg.id,
                            seq: msg.seq,
                            to: self.node_identity.public_key().clone(),
                            from: src_public_key,
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
                OutboundDomainMessage::new(&999, msg),
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
    let mut dir = path.as_ref().to_path_buf();
    dir.pop();
    fs::create_dir_all(dir)?;
    let buf = serde_json::to_vec(item)?;
    File::create(path)?.write_all(&buf)?;
    Ok(())
}

pub fn peer_from_str(s: &str) -> Option<Peer> {
    use tari_crypto::tari_utilities::hex::Hex;
    let mut split = s.splitn(2, "::");
    let pk = split.next().and_then(|s| CommsPublicKey::from_hex(s).ok())?;
    let node_id = NodeId::from_key(&pk);
    let address = split.next().and_then(|s| s.parse::<Multiaddr>().ok())?;
    Some(Peer::new(
        pk,
        node_id,
        vec![address].into(),
        Default::default(),
        PeerFeatures::COMMUNICATION_NODE,
        Default::default(),
        "tari/chess/0.1".to_string(),
    ))
}
