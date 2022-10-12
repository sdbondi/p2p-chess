mod cli;

use std::{env, fs::File, io::Read, path::Path, sync::Arc};

use anyhow::anyhow;
use networking::{Multiaddr, Networking, NetworkingConfig, NodeIdentity, PeerFeatures};
use rand::rngs::OsRng;
use tari_shutdown::Shutdown;
use ui::{ChessUi, ScaleMode, WindowOptions};

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = cli::init();
    #[allow(deprecated)]
    let base_path = cli
        .base_dir
        .clone()
        .or_else(|| env::home_dir())
        .map(|p| p.join(".p2pchess"))
        .ok_or_else(|| anyhow!("Unable to determine home directory. Use --base-dir to specify one."))?;
    let node_identity = load_json(base_path.join("node-identity.json"))?
        .map(Arc::new)
        .unwrap_or_else(create_node_identity);
    let shutdown = Shutdown::new();
    let signal = shutdown.to_signal();

    let (channel1, channel2) = p2p_chess_channel::channel(10);
    let ui = ChessUi::new(
        "Privacy Chess",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions {
            title: true,
            scale_mode: ScaleMode::Center,
            resize: true,
            ..Default::default()
        },
        base_path.clone(),
        channel1,
        node_identity.public_key().clone(),
    );

    let config = NetworkingConfig {
        start_inprocess_tor: cli.local_tor_control_port.is_none(),
        tor_control_port: cli.local_tor_control_port,
    };
    println!("Starting networking...");
    let mut networking = Networking::start(config, node_identity, &base_path, channel2, signal).await?;

    loop {
        println!("Waiting for peer connections...");
        match networking.wait_for_connectivity().await {
            Ok(_) => break,
            Err(e) => println!("{}. Still waiting...", e),
        }
    }

    println!("Starting UI");
    ui.run()?;

    Ok(())
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

fn create_node_identity() -> Arc<NodeIdentity> {
    Arc::new(NodeIdentity::random(
        &mut OsRng,
        Multiaddr::empty(),
        PeerFeatures::COMMUNICATION_CLIENT,
    ))
}
