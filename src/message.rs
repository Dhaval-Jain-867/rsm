use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::transaction::TransactionEnvelope;

#[derive(Serialize, Deserialize)]
pub enum P2pMessage {
    Handshake (String),
    NewTransaction(TransactionEnvelope),
    NewBlock(Block),
    RequestChain,

    RequestPeers(String),
    PeerList(Vec<String>),
}