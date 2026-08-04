use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{println, thread};

use crate::block::{Block, Blockchain};
use crate::message::P2pMessage;
use crate::miner::Miner;
use crate::transaction::TransactionEnvelope;

#[derive(Clone)]
pub struct Node {
    pub address: String,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub peers: Arc<Mutex<Vec<String>>>,
    pub miner_wallet: Miner,
}

impl Node {
    pub fn new(address: String, blockchain: Blockchain) -> Self {
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
            println!("Bootstrapping from seed node: {}", seed_address);

            self.peers.lock().unwrap().push(seed_address.clone());
            self.do_handshake(&seed_address);
            self.broadcast(P2pMessage::RequestPeers(self.address.clone()));
            self.broadcast(P2pMessage::RequestChain(self.address.clone()));
        } else {
            println!("Starting as the genesis node. Waiting for peers...");
        }
    }

    fn start_server(&self) {
        let server_node = self.clone();

        thread::spawn(move || {
            let listener = TcpListener::bind(&server_node.address).expect("Failed to bind");
            println!("Node listening on {}", server_node.address);

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

        println!("Minted a new block");
        self.handle_message(P2pMessage::NewBlock(block));
    }

    fn handle_connection(&mut self, mut stream: TcpStream) {
        println!("Received incoming connection");

        let mut buffer = String::new();
        if stream.read_to_string(&mut buffer).is_ok() {
            if let Ok(message) = serde_json::from_str::<P2pMessage>(&buffer) {
                println!("Handling message");
                self.handle_message(message);
            }
        }
    }

    fn handle_message(&mut self, message: P2pMessage) {
        println!("\n");
        match message {
            P2pMessage::Handshake(address) => {
                println!("Handshake receives from {}", address);
                {
                    let mut peers = self.peers.lock().unwrap();
                    if !peers.contains(&address) {
                        peers.push(address.clone());
                    }
                    println!("Current peers: {:?}", *peers);
                }

                println!("Informing all the peers about new peer: {}", address);
                let message = P2pMessage::NewPeer(address.clone());
                self.broadcast_except(message, address);
            }

            P2pMessage::NewPeer(new_peer) => {
                let mut my_peers = self.peers.lock().unwrap();
                if new_peer != self.address && !my_peers.contains(&new_peer) {
                    my_peers.push(new_peer);
                }
                println!("Peer list updated successfully for the new peer");
                println!("Peer list: {:?}", *my_peers);
            }

            P2pMessage::RequestPeers(peer_address) => {
                println!("Peer: {} requesting peer list", peer_address);

                let peer_list = self.peers.lock().unwrap().clone();
                let message = P2pMessage::PeerList(peer_list);
                let json = serde_json::to_string(&message).unwrap();

                match TcpStream::connect(&peer_address) {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).unwrap();
                        println!("Peer list sent to peer: {}", peer_address)
                    }
                    Err(_e) => {
                        println!("Error connecting to peer: {}", peer_address);
                        println!("Unable to send peer list");
                    }
                }
            }

            P2pMessage::PeerList(peer_list) => {
                println!("Received a peer list");
                let mut my_peers = self.peers.lock().unwrap();

                println!("Updating my peer list");
                for peer in peer_list {
                    if peer != self.address && !my_peers.contains(&peer) {
                        my_peers.push(peer);
                    }
                }

                println!("Current peers: {:?}", *my_peers);
            }

            P2pMessage::RequestChain(peer_address) => {
                println!("Peer: {} requesting chain", peer_address);

                let chain = self.blockchain.lock().unwrap().chain.clone();
                let message = P2pMessage::ChainResponse(chain);
                let json = serde_json::to_string(&message).unwrap();

                match TcpStream::connect(&peer_address) {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).unwrap();
                        println!("Chain sent to peer: {}", peer_address);
                    }
                    Err(_e) => {
                        println!("Error connecting to peer: {}", peer_address);
                        println!("Unable to send chain");
                    }
                }
            }

            P2pMessage::ChainResponse(chain_received) => {
                println!("Recieved a chain");

                if Blockchain::validate_chain(&chain_received) {
                    if chain_received.len() > self.height() {
                        let balance = Blockchain::rebuild_state(&chain_received);
                        match balance {
                            Ok(balance) => {
                                let mut blockchain = self.blockchain.lock().unwrap();
                                blockchain.chain = chain_received;
                                blockchain.balance = balance;
                                println!("Chain updated successfully");
                                println!("Current chain length : {}", blockchain.chain.len());
                            }
                            Err(e) => {
                                println!("Error rebuilding state from chain: {}", e);
                            }
                        }
                    } else {
                        println!(
                            "Length of received chain is shorter than or equal to the current version"
                        );
                    }
                } else {
                    println!("Invalid chain received");
                }
            }

            P2pMessage::SubmitTransaction(tx) => {
                println!("Received new transaction from wallet");

                {
                    let mut my_chain = self.blockchain.lock().unwrap();
                    let mem_add = my_chain.submit_transaction(tx.clone());

                    match mem_add {
                        Ok(_) => {
                            println!("Transaction added to mempool successfully");
                        }
                        Err(e) => {
                            println!("Couldn't enter transaction to mempool: {}", e);
                            return;
                        }
                    }
                    println!(
                        "mempool updated successfully. mempool length: {}",
                        my_chain.mempool.len()
                    );
                }

                println!("Informing all the peers about new transaction");
                let message = P2pMessage::PropagateTransaction(tx);
                self.broadcast(message);
            }

            P2pMessage::PropagateTransaction(tx) => {
                println!("Received new transaction from peer node");

                let mut my_chain = self.blockchain.lock().unwrap();
                let mem_add = my_chain.submit_transaction(tx);

                match mem_add {
                    Ok(_) => {
                        println!("Transaction added to mempool successfully");
                    }
                    Err(e) => {
                        println!("Couldn't enter transaction to mempool: {}", e);
                        return;
                    }
                }

                println!(
                    "mempool updated successfully. mempool length: {}",
                    my_chain.mempool.len()
                );
            }

            P2pMessage::NewBlock(block) => {
                println!("Received block, index: {} from miner", block.index);

                // block will be validated and added to the chain
                {
                    let mut my_chain = self.blockchain.lock().unwrap();
                    let block_added = my_chain.add_block(block.clone());
                    match block_added {
                        Ok(_) => {
                            println!("Block added to chain successfully");
                        }
                        Err(e) => {
                            println!("Couldn't enter block to the chain: {}", e);
                            return;
                        }
                    }
                }

                println!("Informing all the peers about new block");
                let message = P2pMessage::PropagateBlock(block);
                self.broadcast(message);
            }

            P2pMessage::PropagateBlock(block) => {
                println!("Received block, index: {} from peer node", block.index);

                let mut my_chain = self.blockchain.lock().unwrap();
                let block_added = my_chain.add_block(block.clone());

                match block_added {
                    Ok(_) => {
                        println!("Block added to chain successfully");
                    }
                    Err(e) => {
                        println!("Couldn't enter block to the chain: {}", e);
                        return;
                    }
                }
            }
        }
        println!("\n");
    }

    pub fn broadcast(&self, message: P2pMessage) {
        let message_json = serde_json::to_string(&message).expect("Serializtion failed");
        let peers = self.peers.lock().unwrap().clone(); // cloning so that a slow network connection does not block every other thread that wants to access peers

        for peer in peers.iter() {
            if let Ok(mut stream) = TcpStream::connect(peer) {
                let _ = stream.write_all(message_json.as_bytes()).unwrap();
            } else {
                println!("Failed to connect to peer : {}", peer);
            }
        }
    }

    pub fn broadcast_except(&self, message: P2pMessage, address: String) {
        let message_json = serde_json::to_string(&message).expect("Serializtion failed");
        let peers = self.peers.lock().unwrap().clone();

        for peer in peers.iter() {
            if (*peer != address) {
                if let Ok(mut stream) = TcpStream::connect(peer) {
                    let _ = stream.write_all(message_json.as_bytes()).unwrap();
                } else {
                    println!("Failed to connect to peer : {}", peer);
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
                println!("Handshake sent to {}", peer);
            }
            Err(e) => {
                println!("Failed to do handshake: {}", e);
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
