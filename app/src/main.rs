use networking::{Networking, NodeIdentity};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use ui::command::CommandSubscription;
use ui::{ChessUi, ScaleMode, WindowOptions};

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;

fn main() -> anyhow::Result<()> {
    let base_path = PathBuf::from_str("/tmp/p2pchess")?;
    let node_identity = load_json(base_path.join("node_identity.json"))?
        .map(Arc::new)
        .unwrap_or_else(create_node_identity);

    let mut ui = ChessUi::new(
        "P2P Chess",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions {
            title: true,
            scale_mode: ScaleMode::Center,
            resize: true,
            ..Default::default()
        },
    );

    let mut out_cmds = ui.subscribe();
    // let state = ui.shared_state();

    let runtime = Runtime::new()?;
    let networking = {
        let _e = runtime.enter();
        Networking::spawn(node_identity, &base_path, out_cmds)?
    };

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

fn save_json<T: serde::Serialize, P: AsRef<Path>>(path: P, item: &T) -> anyhow::Result<()> {
    fs::create_dir_all(&path)?;
    let buf = serde_json::to_vec(item)?;
    File::create(path)?.write_all(&buf)?;
    Ok(())
}

fn create_node_identity() -> Arc<NodeIdentity> {
    Arc::new(NodeIdentity::random(
        &mut OsRng,
        Multiaddr::empty(),
        PeerFeatures::COMMUNICATION_CLIENT,
    ))
}
