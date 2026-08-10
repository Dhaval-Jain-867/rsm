use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{fs, println, thread};

use crate::Balance;
use crate::block::{Block, Blockchain};
use crate::message::ClientMessage::RequestBalance;
use crate::message::{ClientMessage, NetworkMessage, P2pMessage};
use crate::miner::Miner;
use crate::transaction::TransactionEnvelope;
use crate::wallet::Wallet;

use sha2::digest::consts::True;
use tracing::{error, info, info_span, instrument, warn};

#[derive(Clone)]
pub struct Node {
    pub address: String,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub peers: Arc<Mutex<Vec<String>>>,
    pub miner_wallet: Miner,
}

impl Node {
    pub fn start_cli(&mut self) {
        loop {
            print!("node>");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            self.execute_command(input);
        }
    }

    fn execute_command(&mut self, input: &str) {
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap();

        match command {
            "help" => {
                println!("Available commands -");
                println!("info - Show blockchain status");
                println!("chain - Get entire blockchain");
                println!("peers - List connected peers");
                println!("mine - Mines a block");
                println!("exit - Shut down the node");
            }
            "info" => {
                self.display_info();
            }
            "chain" => {
                self.display_chain();
            }
            "peers" => {
                let peers = self.peers.lock().unwrap();
                println!("Connected to {} peers : {:?}", peers.len(), *peers);
            }
            "mine" => {
                self.mine_new_block();
            }
            "mempool" => {
                let my_chain = self.blockchain.lock().unwrap();
                println!("Mempool size: {}", my_chain.mempool.len());

                for tx in &my_chain.mempool {
                    println!(
                        "TX: {} -> {}: {}",
                        hex::encode(tx.payload.payer),
                        hex::encode(tx.payload.receiver),
                        tx.payload.amount
                    );
                }
            }
            "exit" => {
                println!("Shutting down the node");
                std::process::exit(0);
            }
            _ => {
                println!("Unknown command: {}", command);
            }
        }
    }

    pub fn bootstrap(address: String, seed_node: Option<String>) -> Self {
        let node: Node = Node::new(address, seed_node.clone());
        node.start(seed_node);
        return node;
    }

    pub fn new(address: String, seed_node: Option<String>) -> Self {
        let blockchain;
        match seed_node {
            Some(_) => {
                blockchain = Blockchain::empty();
            }
            None => {
                let g_wallet;
                (blockchain, g_wallet) = Blockchain::new(1000000).unwrap();

                g_wallet.save_to_disk("wallets/faucet.json");
            }
        };
        Self {
            address,
            blockchain: Arc::new(Mutex::new(blockchain)),
            peers: Arc::new(Mutex::new(Vec::new())),
            miner_wallet: Miner::new(),
        }
    }

    pub fn start(&self, seed_node: Option<String>) {
        self.start_server();

        if let Some(seed_address) = seed_node {
            info!(seed = %seed_address, "Bootstrapping from seed node");

            self.peers.lock().unwrap().push(seed_address.clone());
            self.do_handshake(&seed_address);
            self.broadcast(NetworkMessage::P2p(P2pMessage::RequestPeers(
                self.address.clone(),
            )));
            self.broadcast(NetworkMessage::P2p(P2pMessage::RequestChain(
                self.address.clone(),
            )));
        } else {
            info!("Starting as the genesis node. Waiting for peers...");
        }
    }

    fn start_server(&self) {
        let server_node = self.clone();

        thread::spawn(move || {
            let listener = TcpListener::bind(&server_node.address).expect("Failed to bind");
            info!(address = %server_node.address, "Node listening for P2P connections");

            for stream in listener.incoming() {
                if let Ok(tcp_stream) = stream {
                    let mut handler_node = server_node.clone();
                    thread::spawn(move || {
                        handler_node.handle_connection(tcp_stream);
                    });
                }
            }
        });
    }

    fn mine_new_block(&mut self) {
        println!("Mining");
        let block;
        {
            let mut my_chain = self.blockchain.lock().unwrap();
            block = self.miner_wallet.mine_block(&mut my_chain);
        }

        match block {
            Ok(b) => {
                info!(block_index = b.index, "Successfully mined a new block");
                println!("Block #{} mined successfully", b.index);
                self.handle_message(NetworkMessage::P2p(P2pMessage::NewBlock(b)), None);
            }
            Err(e) => {
                error!("Couldn't mine a new block: {}", e);
                println!("Error mining new block. check logs");
            }
        }
    }

    fn display_info(&self) {
        {
            let chain = self.blockchain.lock().unwrap();

            println!("NODE INFORMATION");
            println!("Chain Height      : {}", chain.chain.len());
            println!("Mempool Size      : {}", chain.mempool.len());
            println!("Connected Peers   : {}", self.peers.lock().unwrap().len());

            if let Some(last_block) = chain.chain.last() {
                println!("Latest Block      : {}", last_block.index);
                println!("Latest Hash       : {}", last_block.hash);
            }

            println!(
                "Miner Address     : {}",
                hex::encode(self.miner_wallet.public_key)
            );
        }
    }

    fn display_chain(&self) {
        let chain = self.blockchain.lock().unwrap();

        println!("BLOCKCHAIN");

        for block in &chain.chain {
            println!("------------------------------");
            println!("Block #{}", block.index);
            println!("Hash         : {}", block.hash);
            println!("Previous Hash: {}", block.previous_hash);
            println!("Transactions : {}", block.data.len());
            println!("Reward       : {}", block.reward.amount);
        }
    }

    fn display_peers(&self) {
        let peers = self.peers.lock().unwrap();

        println!("Total peers: {}", peers.len());
        println!("CONNECTED PEERS");

        for peer in peers.iter() {
            println!("{}", peer);
        }
    }

    fn handle_connection(&mut self, mut stream: TcpStream) {
        let peer_ip = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let _conn_span = info_span!("tcp_rx", ip=%peer_ip).entered();

        let mut buffer = String::new();
        if stream.read_to_string(&mut buffer).is_ok() {
            if let Ok(message) = serde_json::from_str::<NetworkMessage>(&buffer) {
                self.handle_message(message, Some(&mut stream));
            } else {
                warn!("Received unparseable garbage data over TCP");
            }
        }
    }

    #[instrument(skip(self, message))]
    fn handle_message(&mut self, message: NetworkMessage, stream: Option<&mut TcpStream>) {
        match message {
            NetworkMessage::P2p(P2pMessage::Handshake(address)) => {
                let _span = info_span!("handshake", peer = %address).entered();
                info!("Handshake received");
                {
                    let mut peers = self.peers.lock().unwrap();
                    if !peers.contains(&address) {
                        peers.push(address.clone());
                    }
                    info!(total_peers = peers.len(), "Peer list updated");
                }

                let message = NetworkMessage::P2p(P2pMessage::NewPeer(address.clone()));
                self.broadcast_except(message, address);
            }

            NetworkMessage::P2p(P2pMessage::NewPeer(new_peer)) => {
                let mut my_peers = self.peers.lock().unwrap();
                if new_peer != self.address && !my_peers.contains(&new_peer) {
                    my_peers.push(new_peer.clone());
                }
                info!(new_peer = %new_peer, total_peers = my_peers.len(), "Added new peer to directory");
            }

            NetworkMessage::P2p(P2pMessage::RequestPeers(peer_address)) => {
                info!(requester = %peer_address, "Sending peer list");

                let peer_list = self.peers.lock().unwrap().clone();
                let message = NetworkMessage::P2p(P2pMessage::PeerList(peer_list));
                let json = serde_json::to_string(&message).unwrap();

                match TcpStream::connect(&peer_address) {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).unwrap();
                    }
                    Err(_e) => {
                        warn!(peer = %peer_address, "Failed to connect to send peer list");
                    }
                }
            }

            NetworkMessage::P2p(P2pMessage::PeerList(peer_list)) => {
                info!(
                    received_count = peer_list.len(),
                    "Received peer list from network"
                );
                let mut my_peers = self.peers.lock().unwrap();

                for peer in peer_list {
                    if peer != self.address && !my_peers.contains(&peer) {
                        my_peers.push(peer);
                    }
                }
            }

            NetworkMessage::P2p(P2pMessage::RequestChain(peer_address)) => {
                info!(requester = %peer_address, "Serving chain state");

                let chain = self.blockchain.lock().unwrap().chain.clone();
                let message = NetworkMessage::P2p(P2pMessage::ChainResponse(chain));
                let json = serde_json::to_string(&message).unwrap();

                match TcpStream::connect(&peer_address) {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).unwrap();
                    }
                    Err(_e) => {
                        warn!(peer = %peer_address, "Failed to connect to serve chain");
                    }
                }
            }

            NetworkMessage::P2p(P2pMessage::ChainResponse(chain_received)) => {
                let _span = info_span!("sync chain").entered();
                info!(
                    received_height = chain_received.len(),
                    "Received chain state"
                );

                if Blockchain::validate_chain(&chain_received) {
                    if chain_received.len() > self.height() {
                        let balance_result = Blockchain::rebuild_state(&chain_received);
                        match balance_result {
                            Ok(balance) => {
                                
                                {
                                    let mut blockchain = self.blockchain.lock().unwrap();
                                    blockchain.chain = chain_received;
                                    blockchain.balance = balance;
                                }
                                info!("Chain updated successfully");
                                info!("Requesting mempool from peers");
                                self.broadcast(NetworkMessage::P2p(P2pMessage::RequestMempool(
                                    self.address.clone(),
                                )));
                            }
                            Err(e) => {
                                error!("Failed to rebuild state from valid chain: {}", e)
                            }
                        }
                    } else {
                        info!("Received chain is shorter or equal to our version. Ignored.");
                        info!("Requesting mempool from peers");
                        self.broadcast(NetworkMessage::P2p(P2pMessage::RequestMempool(
                            self.address.clone(),
                        )));
                    }
                } else {
                    warn!("Received invalid chain data from peer");
                }
            }

            NetworkMessage::P2p(P2pMessage::RequestMempool(peer_address)) => {
                info!(requester = %peer_address, "Sending mempool state to: {}", peer_address);

                let mempool_data: Vec<TransactionEnvelope> = {
                    let chain = self.blockchain.lock().unwrap();
                    chain.mempool.iter().cloned().collect()
                };

                let message = NetworkMessage::P2p(P2pMessage::MempoolResponse(mempool_data));
                let json = serde_json::to_string(&message).unwrap();

                match TcpStream::connect(&peer_address) {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).unwrap();
                    }
                    Err(e) => {
                        warn!(peer = %peer_address, "Failed to connect to node to send mempool");
                    }
                }
            }

            NetworkMessage::P2p(P2pMessage::MempoolResponse(mempool_data)) => {
                let _span = info_span!("sync_mempool").entered();

                info!(
                    received_tx_count = mempool_data.len(),
                    "Received mempool data from peer"
                );

                let mut my_chain = self.blockchain.lock().unwrap();
                let mut added_count = 0;

                for tx in mempool_data {
                    if my_chain.submit_transaction(tx).is_ok() {
                        added_count += 1;
                    }
                }

                info!(
                    added = added_count,
                    current_mempool_size = my_chain.mempool.len(),
                    "Mempool synchronization complete"
                );
            }

            NetworkMessage::P2p(P2pMessage::PropagateTransaction(tx)) => {
                let _span = info_span!("tx_network").entered();

                let mut my_chain = self.blockchain.lock().unwrap();
                let mem_add = my_chain.submit_transaction(tx);

                match mem_add {
                    Ok(_) => {
                        info!(
                            mempool_size = my_chain.mempool.len(),
                            "Relayed transaction added to mempool"
                        );
                    }
                    Err(e) => {
                        warn!("Relayed transaction rejected: {}", e);
                        return;
                    }
                }
            }

            NetworkMessage::P2p(P2pMessage::NewBlock(block)) => {
                let _span = info_span!("block_rx", index = block.index).entered();
                info!("Received new block from local miner");
                {
                    let mut my_chain = self.blockchain.lock().unwrap();
                    let block_added = my_chain.add_block(block.clone());
                    match block_added {
                        Ok(_) => {
                            info!("Block verified and added to chain");
                        }
                        Err(e) => {
                            warn!("Block rejected: {}", e);
                            return;
                        }
                    }
                }
                let message = NetworkMessage::P2p(P2pMessage::PropagateBlock(block));
                self.broadcast(message);
            }

            NetworkMessage::P2p(P2pMessage::PropagateBlock(block)) => {
                let _span = info_span!("block_rx", index = block.index).entered();
                info!("Received block from peer network");

                let mut my_chain = self.blockchain.lock().unwrap();
                let block_added = my_chain.add_block(block.clone());

                match block_added {
                    Ok(_) => {
                        info!("Network block verified and added to chain");
                    }
                    Err(e) => {
                        warn!("Network block rejected: {}", e);

                        if e == "Block does not extend current chain" {
                            info!("Block doesn't fit our chain. Requesting full chain sync..");

                            let req = NetworkMessage::P2p(P2pMessage::RequestChain(self.address.clone()));
                            drop(my_chain);
                            self.broadcast(req);
                        }

                        return;
                    }
                }
            }

            NetworkMessage::Client(ClientMessage::SubmitTransaction(tx)) => {
                let _span = info_span!("tx_local").entered();
                info!("Received new transaction from wallet");

                let response_msg;
                {
                    let mut my_chain = self.blockchain.lock().unwrap();
                    let mem_add = my_chain.submit_transaction(tx.clone());

                    match mem_add {
                        Ok(_) => {
                            info!(mempool_size = my_chain.mempool.len(), "Added to mempool");
                            response_msg = ClientMessage::TransactionResponse {
                                success: true,
                                message: "Transaction added to mempool successfully!".to_string(),
                            };
                        }
                        Err(e) => {
                            warn!("Transaction rejected: {}", e);
                            response_msg = ClientMessage::TransactionResponse {
                                success: false,
                                message: format!("Transaction rejected: {}", e),
                            };
                        }
                    }
                }

                if let Some(s) = stream {
                    let response_network_msg = NetworkMessage::Client(response_msg.clone());
                    let json = serde_json::to_string(&response_network_msg).unwrap();
                    let _ = s.write_all(json.as_bytes());
                } else {
                    warn!("Couldn't respond to wallet");
                }

                if let ClientMessage::TransactionResponse { success: true, .. } = response_msg {
                    let broadcast_message =
                        NetworkMessage::P2p(P2pMessage::PropagateTransaction(tx));
                    self.broadcast(broadcast_message);
                }
            }

            NetworkMessage::Client(ClientMessage::RequestAirdrop(tx)) => {
                let _span = info_span!("airdrop_local").entered();
                info!("Received airdrop request from wallet");

                let response_msg;
                {
                    let mut my_chain = self.blockchain.lock().unwrap();
                    let mem_add = my_chain.submit_transaction(tx.clone());

                    match mem_add {
                        Ok(_) => {
                            info!(mempool_size = my_chain.mempool.len(), "Added to mempool");
                            response_msg = ClientMessage::AirdropResponse {
                                success: true,
                                message: "Transaction added to mempool successfully!".to_string(),
                            };
                        }
                        Err(e) => {
                            warn!("Transaction rejected: {}", e);
                            response_msg = ClientMessage::AirdropResponse {
                                success: false,
                                message: format!("Airdrop rejected: {}", e),
                            };
                        }
                    }
                }

                if let Some(s) = stream {
                    let response_network_msg = NetworkMessage::Client(response_msg.clone());
                    let json = serde_json::to_string(&response_network_msg).unwrap();
                    let _ = s.write_all(json.as_bytes());
                } else {
                    warn!("Couldn't respond to faucet");
                }

                if let ClientMessage::AirdropResponse { success: true, .. } = response_msg {
                    let broadcast_message =
                        NetworkMessage::P2p(P2pMessage::PropagateTransaction(tx));
                    self.broadcast(broadcast_message);
                }
            }

            NetworkMessage::Client(RequestBalance(pubkey)) => {
                info!("Received balance request from wallet");

                let response_msg;
                {
                    let mut my_chain = self.blockchain.lock().unwrap();
                    let balance = my_chain.balance.get_balance(pubkey);
                    response_msg = NetworkMessage::Client(ClientMessage::BalanceResponse(balance));
                }

                if let Some(s) = stream {
                    let json = serde_json::to_string(&response_msg).unwrap();
                    let _ = s.write_all(json.as_bytes());
                } else {
                    warn!("Couldn't respond to wallet");
                }
            }

            NetworkMessage::Client(_) => {
                warn!("Node received unexpected Client message");
            }
        }
    }

    #[instrument(skip(self, message))]
    pub fn broadcast(&self, message: NetworkMessage) {
        let message_json = serde_json::to_string(&message).expect("Serializtion failed");
        let peers = self.peers.lock().unwrap().clone(); // cloning so that a slow network connection does not block every other thread that wants to access peers

        for peer in peers.iter() {
            if let Ok(mut stream) = TcpStream::connect(peer) {
                let _ = stream.write_all(message_json.as_bytes()).unwrap();
            } else {
                warn!(peer = %peer, "Failed to connect during broadcast");
            }
        }
    }

    #[instrument(skip(self, message))]
    pub fn broadcast_except(&self, message: NetworkMessage, address: String) {
        let message_json = serde_json::to_string(&message).expect("Serializtion failed");
        let peers = self.peers.lock().unwrap().clone();

        for peer in peers.iter() {
            if (*peer != address) {
                if let Ok(mut stream) = TcpStream::connect(peer) {
                    let _ = stream.write_all(message_json.as_bytes()).unwrap();
                } else {
                    warn!(peer = %peer, "Failed to connect during targeted broadcast")
                }
            }
        }
    }

    pub fn do_handshake(&self, peer: &str) {
        let message = NetworkMessage::P2p(P2pMessage::Handshake(self.address.clone()));
        let json = serde_json::to_string(&message).unwrap();

        match TcpStream::connect(peer) {
            Ok(mut stream) => {
                stream.write_all(json.as_bytes()).unwrap();
                info!(peer = %peer, "Handshake sent");
            }
            Err(e) => {
                error!(peer = %peer, error = %e, "Failed to perform handshake");
            }
        }
    }

    pub fn height(&self) -> usize {
        let b_chain = self.blockchain.lock().unwrap();
        b_chain.chain.len()
    }
}
