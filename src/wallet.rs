use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::{
    env, fs,
    io::{self, Read, Write},
    net::{Shutdown, TcpStream},
    path::Path,
    print, println,
};

use borsh;
use ed25519_dalek::{Signer, SigningKey};
use getrandom;
use hex;

use crate::hash;
use crate::message::{ClientMessage, NetworkMessage};
use crate::transaction::{Transaction, TransactionEnvelope};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: [u8; 32],
    private_key: [u8; 32],
}

impl Wallet {
    pub fn start_cli(&self, is_faucet: bool) {
        if is_faucet {
            loop {
                print!("faucet>");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                self.execute_faucet_command(input);
            }
        } else {
            loop {
                print!("wallet>");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                self.execute_wallet_command(input);
            }
        }
    }

    fn execute_faucet_command(&self, input: &str) {
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap();

        match command {
            "help" => {
                println!("Available commands -");
                println!("airdrop - get some tokens to a wallet");
            }
            "airdrop" => {
                let amount_str = parts.next();
                let receiver_str = parts.next();
                let node_addr = parts.next().unwrap_or("127.0.0.1:8001");

                if let (Some(amt), Some(recv)) = (amount_str, receiver_str) {
                    if let Ok(amount) = amt.parse::<u64>() {
                        println!("Creating airdrop transaction");
                        let receiver_bytes = hash::public_key_from_string(recv);
                        match receiver_bytes {
                            Ok(bytes) => {
                                let new_transaction = self.create_transaction(bytes, amount);
                                let msg = NetworkMessage::Client(ClientMessage::RequestAirdrop(
                                    new_transaction,
                                ));
                                self.airdrop_request(node_addr, msg);
                            }
                            Err(_) => {
                                println!("Error creating transaction");
                            }
                        }
                    } else {
                        println!("Invalid amount. Must be a number");
                    }
                } else {
                    println!("Invalid call");
                }
            }
            _ => {
                println!("Unknown command: {}", command);
            }
        }
    }

    fn execute_wallet_command(&self, input: &str) {
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap();

        match command {
            "help" => {
                println!("Available commands -");
                println!("info - Show public & private key");
            }
            "info" => {
                println!("Public key: {}", self.get_public_key());
                println!("Private key: {}", self.get_private_key());
            }
            "balance" => {
                let node_addr = parts.next().unwrap_or("127.0.0.1:8001");
            }
            "send" => {
                let amount_str = parts.next();
                let receiver_str = parts.next();
                let node_addr = parts.next().unwrap_or("127.0.0.1:8001");

                if let (Some(amt), Some(recv)) = (amount_str, receiver_str) {
                    if let Ok(amount) = amt.parse::<u64>() {
                        println!("Creating Transaction");
                        let receiver_bytes = hash::public_key_from_string(recv);
                        match receiver_bytes {
                            Ok(bytes) => {
                                let new_transaction = self.create_transaction(bytes, amount);
                                let msg = NetworkMessage::Client(ClientMessage::SubmitTransaction(
                                    new_transaction,
                                ));
                                self.send_to_node(node_addr, msg);
                            }
                            Err(_) => {
                                println!("Error creating transaction");
                            }
                        }
                    } else {
                        println!("Invalid amount. Must be a number");
                    }
                } else {
                    println!("Invalid call");
                }
            }
            "balance" => {
                let node_addr = parts.next().unwrap_or("127.0.0.1:8001");

                println!("Fetching balance");
                let msg = NetworkMessage::Client(ClientMessage::RequestBalance(self.public_key));
                self.get_balance(node_addr, msg);
            }
            "save" => {
                let wallet_name = parts.next();
                match wallet_name {
                    Some(wn) => {
                        let final_addr = format!("wallets/{}.json", wn);
                        self.save_to_disk(&final_addr);
                    }
                    None => {
                        println!("Command requires wallet save name as well");
                    }
                }
            }
            _ => {
                println!("Unknown command: {}", command);
            }
        }
    }

    fn get_balance(&self, node_addr: &str, msg: NetworkMessage) {
        let json = serde_json::to_string(&msg).unwrap();
        match TcpStream::connect(node_addr) {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(json.as_bytes()) {
                    println!("Failed to connect to node over stream");
                    return;
                }
                stream.shutdown(Shutdown::Write).unwrap();
                println!("Balance requested");

                let mut buffer = String::new();
                if stream.read_to_string(&mut buffer).is_ok() {
                    if let Ok(NetworkMessage::Client(ClientMessage::BalanceResponse(b))) = serde_json::from_str::<NetworkMessage>(&buffer)
                    {
                        println!("Balance: {}", b);
                    } else {
                        println!("Received unknown response from node");
                    }
                } else {
                    println!("Received no response from node");
                }
            }
            Err(e) => {
                println!("Error connecting to node: {}", e);
            }
        }
    }

    fn airdrop_request(&self, node_addr: &str, msg: NetworkMessage) {
        let json = serde_json::to_string(&msg).unwrap();
        match TcpStream::connect(node_addr) {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(json.as_bytes()) {
                    println!("Failed to send transaction to node over stream");
                    return;
                }
                println!("Airdrop requested");

                let mut buffer = String::new();
                if stream.read_to_string(&mut buffer).is_ok() {
                    if let Ok(NetworkMessage::Client(ClientMessage::AirdropResponse {
                        success,
                        message,
                    })) = serde_json::from_str::<NetworkMessage>(&buffer)
                    {
                        if success {
                            println!("Airdrop successful");
                        } else {
                            println!("Airdrop failed");
                        }
                    } else {
                        println!("Received unknown response from node");
                    }
                } else {
                    println!("Received no response from node");
                }
            }
            Err(e) => {
                println!("Error connecting to node: {}", e);
            }
        }
    }

    fn send_to_node(&self, node_addr: &str, msg: NetworkMessage) {
        let json = serde_json::to_string(&msg).unwrap();
        match TcpStream::connect(node_addr) {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(json.as_bytes()) {
                    println!("Failed to send transaction to node over stream");
                    return;
                }
                println!("Transaction sent");

                let mut buffer = String::new();
                if stream.read_to_string(&mut buffer).is_ok() {
                    if let Ok(NetworkMessage::Client(ClientMessage::TransactionResponse {
                        success,
                        message,
                    })) = serde_json::from_str::<NetworkMessage>(&buffer)
                    {
                        if success {
                            println!("Transaction accepted");
                        } else {
                            println!("Transaction rejected")
                        }
                    } else {
                        println!("Received unknown response from node");
                    }
                } else {
                    println!("Received no response from node");
                }
            }
            Err(e) => {
                println!("Error connecting to node: {}", e);
            }
        }
    }

    pub fn new() -> Self {
        let mut secret_bytes = [0u8; 32];
        getrandom::fill(&mut secret_bytes).expect("OS failed to generate random bytes");

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let public_key = signing_key.verifying_key();

        Self {
            public_key: public_key.to_bytes(),
            private_key: secret_bytes,
        }
    }

    pub fn get_public_key(&self) -> String {
        hex::encode(self.public_key)
    }

    fn get_private_key(&self) -> String {
        hex::encode(self.private_key)
    }

    pub fn sign_and_hash_transaction(&self, payload: Transaction) -> TransactionEnvelope {
        let payload_bytes = borsh::to_vec(&payload).expect("Failed to serialize payload");

        // signing the transaction
        let signing_key = SigningKey::from_bytes(&self.private_key);
        let signature_object = signing_key.sign(&payload_bytes);
        let signature_bytes = signature_object.to_bytes();

        // hashing the transaction
        let mut hasher = Sha256::new();
        hasher.update(&payload_bytes);
        let tx_id = hex::encode(hasher.finalize());

        TransactionEnvelope {
            id: tx_id,
            payload: payload,
            signature: signature_bytes,
        }
    }

    pub fn create_transaction(&self, to: [u8; 32], amount: u64) -> TransactionEnvelope {
        let payload = Transaction {
            payer: self.public_key,
            receiver: to,
            amount: amount,
            fees: (amount * env::var("FEES_PERCENT").unwrap().parse::<u64>().unwrap()) / 100,
        };

        let transaction = self.sign_and_hash_transaction(payload);
        return transaction;
    }

    pub fn save_to_disk(&self, file_path: &str) {
        let json_data = serde_json::to_string_pretty(&self).expect("Failed to serialize wallet");
        fs::write(file_path, json_data).expect("Failed to save faucet wallet");
        println!("Faucet wallet saved successfully");
    }

    pub fn load_from_disk(file_path: &str) -> Result<Self, Box<dyn Error>> {
        if Path::new(file_path).exists() {
            let json_data = fs::read_to_string(file_path)?;
            let wallet: Wallet = serde_json::from_str(&json_data)?;
            return Ok(wallet);
        }
        return Err("Unable to load wallet from disk".into());
    }
}
