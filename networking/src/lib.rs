mod message;
mod node;
mod tor_handle;

use std::{
    fs,
    fs::File,
    io,
    io::{Read, Write},
    path::Path,
    process::Command,
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use p2p_chess_channel::{ChessOperation, MessageChannel, OperationType};
use rand::{rngs::OsRng, thread_rng, Rng, RngCore};
// Re-exports
pub use tari_comms::{
    multiaddr::Multiaddr,
    peer_manager::{NodeIdentity, PeerFeatures},
};
use tari_comms::{
    net_address::{MultiaddressesWithStats, PeerAddressSource},
    peer_manager::{NodeId, Peer},
    types::CommsPublicKey,
    CommsNode,
};
use tari_comms_dht::{
    domain_message::OutboundDomainMessage,
    inbound::DecryptedDhtMessage,
    outbound::OutboundEncryption,
    Dht,
};
use tari_shutdown::ShutdownSignal;
use tokio::{sync::mpsc, task};

use crate::{
    message::{Message, MessageType, MoveMsg, NewGameMsg, ProtoMessage, ResignMsg, SyncMsg},
    tor_handle::TorHandle,
};

pub struct Networking {
    dht: Dht,
    in_msg: mpsc::Receiver<DecryptedDhtMessage>,
    channel: MessageChannel<ChessOperation>,
    node_identity: Arc<NodeIdentity>,
}

pub struct NetworkingConfig {
    pub start_inprocess_tor: bool,
    pub tor_control_port: Option<u16>,
}

impl Networking {
    pub async fn start<P: AsRef<Path>>(
        config: NetworkingConfig,
        node_identity: Arc<NodeIdentity>,
        base_path: P,
        channel: MessageChannel<ChessOperation>,
        shutdown_signal: ShutdownSignal,
    ) -> anyhow::Result<NetworkingHandle> {
        fs::create_dir_all(base_path.as_ref())?;
        let tor_identity = load_json(base_path.as_ref().join("tor.json"))?;
        // TODO
        let seed_peers = [
            "881d8742d4cdf5dc99def7271405e6a3fc56080ea3387cc568b9efdef1cdeb7b::/onion3/\
             spxhershjnwrl5p366xx2lc44sjxrhu3kypjj6ws3jxe4yiwy5ocftqd:18141",
        ]
        .into_iter()
        .map(|s| peer_from_str(s).unwrap())
        .collect();

        let control_port = config
            .tor_control_port
            .unwrap_or_else(|| OsRng.gen_range(15000u16..50000));
        let socks_port = OsRng.gen_range(15000u16..50000);

        let mut tor_handle = None;
        if config.start_inprocess_tor {
            let mut handle = start_tor(control_port, socks_port)?;
            handle.wait_for_bootstrap()?;
            tor_handle = Some(handle);
        }

        let port = thread_rng().gen_range(15000..50000);
        let (node, dht, in_msg) = node::create(
            node_identity.clone(),
            base_path.as_ref().join("db"),
            control_port,
            tor_identity,
            port,
            seed_peers,
            shutdown_signal,
        )
        .await?;
        let last_address = node.node_identity().public_addresses().last().unwrap().clone();
        node.node_identity().set_public_addresses(vec![last_address]);
        save_json(base_path.as_ref().join("node-identity.json"), node.node_identity_ref())?;

        let node_identity = node.node_identity();
        let mut handle = NetworkingHandle::new(node);
        handle.tor_handle = tor_handle;

        let worker = Self {
            dht,
            in_msg,
            channel,
            node_identity,
        };
        worker.spawn();

        Ok(handle)
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
        dbg!("sending", &op);
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
            OperationType::Sync { board } => {
                self.broadcast_msg(
                    op.to,
                    Message::new(op.game_id, op.seq, MessageType::Sync, SyncMsg { board }),
                )
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
                dbg!("inbound", &msg_type);
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
                    MessageType::Sync => {
                        let msg = Message::<SyncMsg>::try_from(msg)?;
                        ChessOperation {
                            game_id: msg.id,
                            seq: msg.seq,
                            to: self.node_identity.public_key().clone(),
                            from: src_public_key,
                            operation: OperationType::Sync {
                                board: msg.payload.board,
                            },
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
        let num = thread_rng().next_u32() as i32;
        self.dht
            .outbound_requester()
            .broadcast(
                public_key.clone().into(),
                OutboundEncryption::EncryptFor(Box::new(public_key.clone())),
                vec![],
                OutboundDomainMessage::new(&num, msg.clone()),
                String::new(),
            )
            .await?;

        Ok(())
    }
}

pub struct NetworkingHandle {
    node: CommsNode,
    tor_handle: Option<TorHandle>,
}

impl NetworkingHandle {
    pub fn kill(&mut self) -> io::Result<()> {
        match self.tor_handle.as_mut() {
            Some(handle) => handle.kill(),
            _ => Ok(()),
        }
    }

    pub fn new(node: CommsNode) -> Self {
        Self { node, tor_handle: None }
    }

    pub async fn wait_for_connectivity(&mut self) -> anyhow::Result<()> {
        self.node
            .connectivity()
            .wait_for_connectivity(Duration::from_secs(30))
            .await?;
        Ok(())
    }
}

impl Drop for NetworkingHandle {
    fn drop(&mut self) {
        if let Err(err) = self.kill() {
            log::error!("Error killing tor process: {}", err);
        }
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
    let addresses =
        MultiaddressesWithStats::from_addresses_with_source(vec![address.into()], &PeerAddressSource::Config);
    Some(Peer::new(
        pk,
        node_id,
        addresses,
        Default::default(),
        PeerFeatures::COMMUNICATION_NODE,
        Default::default(),
        "tari/chess/0.1".to_string(),
    ))
}

pub fn start_tor(control_port: u16, socks_port: u16) -> io::Result<TorHandle> {
    let (reader, stdin) = os_pipe::pipe()?;
    let stderr = stdin.try_clone()?;
    let child = Command::new("tor")
        .arg("--controlport")
        .arg(format!("127.0.0.1:{}", control_port))
        .arg("--SocksPort")
        .arg(format!("{}", socks_port))
        .arg("--DataDirectory")
        .arg(".p2pchess/tor")
        .stdout(stdin)
        .stderr(stderr)
        .spawn()?;

    Ok(TorHandle {
        child,
        output: reader,
        control_port,
        socks_port,
    })
}
