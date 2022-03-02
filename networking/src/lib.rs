mod message;
mod node;

use tari_comms::CommsNode;

pub use node::create;
pub use tari_comms::peer_manager::NodeIdentity;

pub struct Networking {
    node: CommsNode,
}
