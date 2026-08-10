use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::transaction::TransactionEnvelope;

#[derive(Serialize, Deserialize)]
pub enum NetworkMessage {
    P2p(P2pMessage),
    Client(ClientMessage)
}

#[derive(Serialize, Deserialize)]
pub enum P2pMessage {
    Handshake (String),

    PropagateTransaction(TransactionEnvelope),

    NewBlock(Block),
    PropagateBlock(Block),

    RequestMempool(String),
    MempoolResponse(Vec<TransactionEnvelope>),

    RequestChain(String),
    ChainResponse(Vec<Block>),

    NewPeer(String),
    RequestPeers(String),
    PeerList(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    SubmitTransaction(TransactionEnvelope),
    RequestAirdrop(TransactionEnvelope),
    TransactionResponse {
        success: bool,
        message: String
    },
    AirdropResponse {
        success: bool,
        message: String
    },
    RequestBalance([u8; 32]),
    BalanceResponse(u64)
}