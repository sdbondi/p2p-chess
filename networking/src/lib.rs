mod handle;
mod message;
mod node;

use rand::rngs::OsRng;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tari_comms::multiaddr::Multiaddr;
use tari_comms::CommsNode;

use crate::handle::NetworkingHandle;
use crate::message::{Message, MessageType, ProtoMessage};
use commands::{ChessOperation, CommandReceiver, OperationType};
pub use tari_comms::peer_manager::NodeIdentity;
use tari_comms::peer_manager::PeerFeatures;
use tari_comms::types::CommsPublicKey;
use tari_comms_dht::domain_message::OutboundDomainMessage;
use tari_comms_dht::inbound::DecryptedDhtMessage;
use tari_comms_dht::outbound::{OutboundEncryption, OutboundMessageRequester};
use tari_comms_dht::Dht;
use tari_shutdown::Shutdown;
use tokio::sync::mpsc;
use tokio::task;

pub struct Networking {
    node: CommsNode,
    dht: Dht,
    in_msg: mpsc::Receiver<DecryptedDhtMessage>,

    outbound_cmds: CommandReceiver,
    inbound_tx: mpsc::Sender<ProstMessage>,
    outbound_rx: mpsc::Receiver<(CommsPublicKey, ProstMessage)>,
    node_identity: Arc<NodeIdentity>,
}

impl Networking {
    pub fn spawn<P: AsRef<Path>>(
        node_identity: Arc<NodeIdentity>,
        base_path: P,
        outbound_cmds: CommandReceiver,
    ) -> anyhow::Result<NetworkingHandle> {
        let shutdown = Shutdown::new();
        let tor_identity = load_json(base_path.as_ref().join("tor.json"))?;
        let seed_peers = vec![];
        let (node, dht, in_msg) = node::create(
            node_identity.clone(),
            base_path.as_ref().join("db"),
            tor_identity,
            9999,
            seed_peers,
            shutdown.to_signal(),
        )
        .await?;
        // let (outbound_tx, outbound_rx) = mpsc::channel(1);
        // let (inbound_tx, inbound_rx) = mpsc::channel(1);

        let worker = Self {
            node,
            dht,
            in_msg,
            outbound_cmds,
            outbound_rx,
            node_identity: node_identity.clone(),
        };
        worker.start();

        Ok(NetworkingHandle {
            outbound_messages: outbound_tx,
            inbound_messages: inbound_rx,
            node_identity,
        })
    }

    fn start(self) {
        task::spawn(self.run_event_loop);
    }

    async fn run_event_loop() {
        loop {
            let res = tokio::select! {
                Some(msg) = self.in_msg.recv() => self.handle_inbound_message(msg).await,
                Some(msg) = self.outbound_cmds.recv() => self.handle_cmd_message(msg).await,
                Some((pk, msg)) = self.outbound_rx.recv() => self.handle_outbound_message(pk, msg).await,
            };
            if let Err(err) = res {
                log::error!("{}", err);
            }
        }
    }

    async fn handle_cmd_message(&self, cmd: ChessOperation) -> anyhow::Result<()> {
        match cmd.operation {
            OperationType::NewGame { player } => {
                self.broadcast_msg(
                    cmd.public_key,
                    Message::new(
                        0,
                        MessageType::NewGame,
                        NewGameMsg {
                            public_key: self.node_identity.public_key().to_vec(),
                            address: self.node_identity.public_address().to_vec(),
                            color: player as u32,
                        },
                    ),
                )
                .await?;
            }
            OperationType::MovePlayed(mv) => {
                // TODO: fetch existing game state
                self.broadcast_msg(
                    cmd.public_key,
                    Message::new(0, MessageType::Move, MoveMsg { val: mv }),
                )
                .await?;
            }
            OperationType::Resign => {}
        }

        Ok(())
    }

    async fn handle_inbound_message(&self, msg: DecryptedDhtMessage) -> anyhow::Result<()> {
        match msg.success() {
            Some(body) => {
                self.inbound_tx.send(body.decode_part(0)).await?;
            }
            None => {
                log::warn!("Received message we could not decrypt {:?}", msg);
            }
        }

        Ok(())
    }

    fn handle_outbound_message(
        &self,
        public_key: CommsPublicKey,
        msg: ProstMessage,
    ) -> anyhow::Result<()> {
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
