use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

use crate::message::P2pMessage;
use crate::block::Blockchain;

#[derive(Clone)]
pub struct Node {
    pub address: String,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub peers: Arc<Mutex<Vec<String>>>,
}

impl Node {
    pub fn new(address: String, blockchain: Blockchain) -> Self {
        Self {
            address,
            blockchain: Arc::new(Mutex::new(blockchain)),
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self, seed_node: Option<String>) {
        self.start_server();

        if let Some(seed_address) = seed_node {
            println!("Bootstrapping from seed node: {}", seed_address);

            self.peers.lock().unwrap().push(seed_address.clone());
            self.do_handshake(&seed_address);
            self.broadcast(P2pMessage::RequestPeers(self.address.clone()));
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
                    let handler_node = server_node.clone();
                    thread::spawn(move || {
                        handler_node.handle_connection(tcp_stream);
                    });
                }
            }
        });
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        println!("Received incoming connection");

        let mut buffer = String::new();

        if stream.read_to_string(&mut buffer).is_ok() {
            if let Ok(message) = serde_json::from_str::<P2pMessage>(&buffer) {
                println!("Received message");
                self.handle_message(message);
            }
        }
    }

    fn handle_message(&self, message: P2pMessage) {
        println!("\n");
        match message {
            P2pMessage::Handshake(address) => {
                println!("Handshake receives from {}", address);

                let mut peers = self.peers.lock().unwrap();

                if !peers.contains(&address) {
                    peers.push(address.clone());
                }

                println!("Current peers: {:?}", *peers);
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

            P2pMessage::RequestChain => {
                println!("Peer requested blockchain");
            }

            P2pMessage::NewTransaction(tx) => {
                println!("Received new transaction");
            }

            P2pMessage::NewBlock(block) => {
                println!("Received new block");
            }
        }
        println!("\n");
    }

    pub fn broadcast(&self, message: P2pMessage) {
        let message_json = serde_json::to_string(&message).expect("Serializtion failed");
        let peers = self.peers.lock().unwrap();

        for peer in peers.iter() {
            if let Ok(mut stream) = TcpStream::connect(peer) {
                let _ = stream.write_all(message_json.as_bytes());
            } else {
                println!("Failed to connect to peer : {}", peer);
            }
        }
    }

    pub fn do_handshake(&self, peer: &str) {
        let message = P2pMessage::Handshake(self.address.clone());
        let json = serde_json::to_string(&message).unwrap();

        match TcpStream::connect(peer) {
            Ok(mut stream) => {
                stream.write_all(json.as_bytes()).unwrap();
                println!("Message sent to {}", peer);
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
}