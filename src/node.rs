use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{println, thread};

use crate::Balance;
use crate::block::{Block, Blockchain};
use crate::message::P2pMessage;
use crate::miner::Miner;
use crate::transaction::TransactionEnvelope;

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

            let mut parts = input.split_whitespace();
            let command = parts.next().unwrap();

            match command {
                "info" => {
                    let chain = self.blockchain.lock().unwrap();
                    println!("Blockchain length: {}", chain.chain.len());
                    println!("Pending transactios in mempool: {}", chain.mempool.len());
                }
                "peers" => {
                    let peers = self.peers.lock().unwrap();
                    println!("Connected to {} peers : {:?}", peers.len(), *peers);
                }
                "help" => {
                    println!("Available commands -");
                    println!("info - Show blockchain status");
                    println!("peers - List connected peers");
                    println!("exit - Shut down the node");
                }
                "exit" => {
                    println!("Shutting down the node");
                    break;
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
    }

    pub fn bootstrap(address: String, seed_node: Option<String>) -> Self {
        let node: Node = Node::new(address, seed_node.clone());
        node.start(seed_node);
        return node;
    }

    pub fn new(address: String, seed_node: Option<String>) -> Self {
        let blockchain = match seed_node {
            Some(_) => Blockchain::empty(),
            None => Blockchain::new(1000).unwrap().0,
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
            self.broadcast(P2pMessage::RequestPeers(self.address.clone()));
            self.broadcast(P2pMessage::RequestChain(self.address.clone()));
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

    pub fn mine_new_block(&mut self) {
        let block;
        {
            let mut my_chain = self.blockchain.lock().unwrap();
            block = self.miner_wallet.mine_block(&mut my_chain).unwrap();
        }

        info!(block_index = block.index, "Successfully minted a new block");
        self.handle_message(P2pMessage::NewBlock(block));
    }

    fn handle_connection(&mut self, mut stream: TcpStream) {
        let peer_ip = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let _conn_span = info_span!("tcp_rx", ip=%peer_ip).entered();

        let mut buffer = String::new();
        if stream.read_to_string(&mut buffer).is_ok() {
            if let Ok(message) = serde_json::from_str::<P2pMessage>(&buffer) {
                self.handle_message(message);
            } else {
                warn!("Received unparseable garbage data over TCP");
            }
        }
    }

    #[instrument(skip(self, message))]
    fn handle_message(&mut self, message: P2pMessage) {
        println!("\n");
        match message {
            P2pMessage::Handshake(address) => {
                let _span = info_span!("handshake", peer = %address).entered();
                info!("Handshake received");
                {
                    let mut peers = self.peers.lock().unwrap();
                    if !peers.contains(&address) {
                        peers.push(address.clone());
                    }
                    info!(total_peers = peers.len(), "Peer list updated");
                }

                let message = P2pMessage::NewPeer(address.clone());
                self.broadcast_except(message, address);
            }

            P2pMessage::NewPeer(new_peer) => {
                let mut my_peers = self.peers.lock().unwrap();
                if new_peer != self.address && !my_peers.contains(&new_peer) {
                    my_peers.push(new_peer.clone());
                }
                info!(new_peer = %new_peer, total_peers = my_peers.len(), "Added new peer to directory");
            }

            P2pMessage::RequestPeers(peer_address) => {
                info!(requester = %peer_address, "Sending peer list");

                let peer_list = self.peers.lock().unwrap().clone();
                let message = P2pMessage::PeerList(peer_list);
                let json = serde_json::to_string(&message).unwrap();

                match TcpStream::connect(&peer_address) {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).unwrap();
                        // println!("Peer list sent to peer: {}", peer_address);
                    }
                    Err(_e) => {
                        warn!(peer = %peer_address, "Failed to connect to send peer list");
                    }
                }
            }

            P2pMessage::PeerList(peer_list) => {
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

            P2pMessage::RequestChain(peer_address) => {
                info!(requester = %peer_address, "Serving chain state");

                let chain = self.blockchain.lock().unwrap().chain.clone();
                let message = P2pMessage::ChainResponse(chain);
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

            P2pMessage::ChainResponse(chain_received) => {
                let _span = info_span!("sync").entered();
                info!(
                    received_height = chain_received.len(),
                    "Received chain state"
                );

                if Blockchain::validate_chain(&chain_received) {
                    if chain_received.len() > self.height() {
                        let balance = Blockchain::rebuild_state(&chain_received);
                        match balance {
                            Ok(balance) => {
                                let mut blockchain = self.blockchain.lock().unwrap();
                                blockchain.chain = chain_received;
                                blockchain.balance = balance;
                                info!(
                                    new_height = blockchain.chain.len(),
                                    "Chain updated successfully"
                                );
                            }
                            Err(e) => {
                                error!("Failed to rebuild state from valid chain: {}", e)
                            }
                        }
                    } else {
                        info!("Received chain is shorter or equal to our version. Ignored.");
                    }
                } else {
                    warn!("Received invalid chain data from peer");
                }
            }

            P2pMessage::SubmitTransaction(tx) => {
                let _span = info_span!("tx_local").entered();
                info!("Received new transaction from local wallet");

                {
                    let mut my_chain = self.blockchain.lock().unwrap();
                    let mem_add = my_chain.submit_transaction(tx.clone());

                    match mem_add {
                        Ok(_) => {
                            info!(mempool_size = my_chain.mempool.len(), "Added to mempool");
                        }
                        Err(e) => {
                            warn!("Transaction rejected: {}", e);
                            return;
                        }
                    }
                    println!(
                        "mempool updated successfully. mempool length: {}",
                        my_chain.mempool.len()
                    );
                }

                let message = P2pMessage::PropagateTransaction(tx);
                self.broadcast(message);
            }

            P2pMessage::PropagateTransaction(tx) => {
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

            P2pMessage::NewBlock(block) => {
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
                let message = P2pMessage::PropagateBlock(block);
                self.broadcast(message);
            }

            P2pMessage::PropagateBlock(block) => {
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
                        return;
                    }
                }
            }
        }
    }

    #[instrument(skip(self, message))]
    pub fn broadcast(&self, message: P2pMessage) {
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
    pub fn broadcast_except(&self, message: P2pMessage, address: String) {
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
        let message = P2pMessage::Handshake(self.address.clone());
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

    // Helpers
    pub fn submit_transaction(&mut self, tx: TransactionEnvelope) {
        self.handle_message(P2pMessage::SubmitTransaction(tx));
    }

    pub fn submit_block(&mut self, block: Block) {
        self.handle_message(P2pMessage::NewBlock(block));
    }
}
