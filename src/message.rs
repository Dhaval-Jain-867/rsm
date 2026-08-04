use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::transaction::TransactionEnvelope;

#[derive(Serialize, Deserialize)]
pub enum P2pMessage {
    Handshake (String),

    SubmitTransaction(TransactionEnvelope),
    PropagateTransaction(TransactionEnvelope),

    NewBlock(Block),
    PropagateBlock(Block),

    RequestChain(String),
    ChainResponse(Vec<Block>),

    NewPeer(String),
    RequestPeers(String),
    PeerList(Vec<String>),
}