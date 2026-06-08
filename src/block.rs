use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;
use std::path::Path;
use std::{env, fs};

use crate::balances::Balance;
use crate::hash;
use crate::transaction::{CoinbaseTransaction, TransactionEnvelope};
use crate::wallet::Wallet;

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u128,
    pub timestamp: i64,
    pub reward: CoinbaseTransaction,
    pub data: Vec<TransactionEnvelope>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u128,
}

pub struct Blockchain {
    pub balance: Balance,
    pub chain: Vec<Block>,
    pub mempool: VecDeque<TransactionEnvelope>,
}

impl Blockchain {
    pub fn new(initial_amount: u64) -> Result<(Self, Wallet), String> {
        let mut bchain = Blockchain {
            balance: Balance::new(),
            chain: Vec::new(),
            mempool: VecDeque::new(),
        };
        let gwallet = bchain.create_genesis_block(initial_amount);
        if let Ok(wallet) = gwallet {
            return Ok((bchain, wallet));
        } else {
            return Err(String::from("Error creating wallet for genesis block"));
        }
    }

    pub fn add_block(&mut self, block: Block, total_count: usize) -> Result<String, String> {
        // safety check but pratically impossible
        if self.chain.is_empty() {
            return Err(String::from(
                "Genesis block should be minted before adding any other block",
            ));
        }
        let mut state_clone = self.balance.clone();

        let mut predicted_reward: u64 = env::var("PER_TX_REWARD").unwrap().parse().unwrap();

        for transaction in &block.data {
            if !transaction.is_valid(&state_clone) {
                return Err(String::from(
                    "An invalid transaction was present in the block",
                ));
            }
            state_clone.transfer(&transaction.payload);
            predicted_reward = predicted_reward
                .checked_add(transaction.payload.fees)
                .unwrap();
        }

        if block.is_valid() && block.reward.amount == predicted_reward {
            *state_clone
                .accounts
                .entry(block.reward.receiver)
                .or_insert(0) += block.reward.amount;
            self.chain.push(block);
            self.balance = state_clone;
            self.mempool.drain(0..total_count);
            return Ok(String::from("Block successfully mined"));
        }

        return Err(String::from(
            "Block is invalid or block reward is incorrect",
        ));
    }

    pub fn create_genesis_block(&mut self, initial_amount: u64) -> Result<Wallet, String> {
        if !self.chain.is_empty() {
            return Err(String::from("Genesis block has already been minted"));
        }
        let genesis_wallet = Wallet::new();
        let coinbase_transaction = CoinbaseTransaction {
            receiver: genesis_wallet.public_key,
            amount: initial_amount,
        };
        let timestamp = Utc::now().timestamp();
        let block_data = Vec::new();
        let mut new_block = Block::new(
            0,
            timestamp,
            coinbase_transaction,
            block_data,
            String::from("0000000000000000000000000000000000000000000000000000000000000000"),
        );
        let final_block = hash::hash_block(&mut new_block);

        // self.add_block(*final_block, 0);
        if final_block.is_valid() {
            *self
                .balance
                .accounts
                .entry(final_block.reward.receiver)
                .or_insert(0) += final_block.reward.amount;
            self.chain.push(final_block.clone());
            return Ok(genesis_wallet);
        }
        return Err(String::from(
            "An error occurred while creating the genesis block",
        ));
    }

    pub fn is_valid(&self) -> bool {
        let chain_length = self.chain.len();
        if chain_length == 0 || self.chain[0].index != 0 {
            return false;
        }
        for i in 0..chain_length {
            let curr_block = &self.chain[i];
            if i != 0 {
                if curr_block.previous_hash == (&self.chain[i - 1]).hash && curr_block.is_valid() {
                    continue;
                } else {
                    return false;
                }
            } else {
                if curr_block.is_valid() {
                    continue;
                } else {
                    return false;
                }
            }
        }
        return true;
    }

    // pub fn mint(&mut self, amount: u64, address: [u8; 32]) {
    //     *self.balance.accounts.entry(address).or_insert(0) += amount;
    // }

    pub fn submit_transaction(
        &mut self,
        transaction_envelope: TransactionEnvelope,
    ) -> Result<String, String> {
        if transaction_envelope.is_valid(&self.balance.clone()) {
            self.mempool.push_back(transaction_envelope);
            return Ok(String::from("Transaction added to mempool"));
        }
        return Err(String::from(
            "Can't enter mempool since transaction was invalid",
        ));
    }

    pub fn rebuild_state(chain: &Vec<Block>) -> Result<Balance, String> {
        let mut new_state = Balance::new();
        for block in chain.iter() {
            let miner_address = block.reward.receiver;
            let mut miner_award: u64 = env::var("PER_TX_REWARD").unwrap().parse().unwrap();

            if block.index == 0 {
                if block.previous_hash
                    != "0000000000000000000000000000000000000000000000000000000000000000"
                {
                    return Err(String::from("Invalid genesis block"));
                }
                *new_state.accounts.entry(block.reward.receiver).or_insert(0) +=
                    block.reward.amount;
                continue;
            }

            for transaction in block.data.iter() {
                if !transaction.is_valid(&new_state) {
                    return Err(String::from("Transaction is invalid"));
                }
                let amount = transaction.payload.amount;
                let fees = transaction.payload.fees;

                if let Some(balance) = new_state.accounts.get_mut(&transaction.payload.payer) {
                    *balance -= amount.checked_add(fees).unwrap();
                } else {
                    return Err(String::from("Payer didn't exist"));
                }
                *new_state
                    .accounts
                    .entry(transaction.payload.receiver)
                    .or_insert(0) += amount;

                if let Some(value) = miner_award.checked_add(transaction.payload.fees) {
                    miner_award = value;
                } else {
                    return Err(String::from("Overflow error"));
                }
            }

            if miner_award != block.reward.amount {
                return Err(String::from("Block reward is invalid"));
            }
            *new_state.accounts.entry(miner_address).or_insert(0) += miner_award;
        }
        Ok(new_state)
    }

    pub fn save_chain_to_disk(&self, file_path: &str) {
        let json_data =
            serde_json::to_string_pretty(&self.chain).expect("Failed to serialize blockchain");
        fs::write(file_path, json_data).expect("Failed to write blockchain data to dis");
        println!("Blockchain saved successfully to disk");
    }

    pub fn load_chain_from_disk(file_path: &str) -> Result<Self, Box<dyn Error>> {
        if Path::new(file_path).exists() {
            let json_data = fs::read_to_string(file_path)?;
            let chain: Vec<Block> = serde_json::from_str(&json_data)?;
            let balance = Blockchain::rebuild_state(&chain).unwrap();
            let mempool: VecDeque<TransactionEnvelope> = VecDeque::new();
            let blockchain = Self {
                balance: balance,
                chain: chain,
                mempool: mempool,
            };

            if !blockchain.is_valid() {
                return Err("Loaded blockchain was invalid".into());
            }
            return Ok(blockchain);
        }
        return Err("Unable to load chain from disk".into());
    }
}

impl Block {
    pub fn new(
        index: u128,
        timestamp: i64,
        reward: CoinbaseTransaction,
        data: Vec<TransactionEnvelope>,
        previous_hash: String,
    ) -> Self {
        Self {
            index: index,
            timestamp: timestamp,
            reward: reward,
            data: data,
            previous_hash: previous_hash,
            hash: String::from(""),
            nonce: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        let nonce_difficulty = env::var("NONCE_DIFFICULTY").unwrap().parse().unwrap();
        if self.hash == self.calculate_hash(self.nonce)
            && hash::is_hash_valid(&self.hash, nonce_difficulty)
        {
            return true;
        }
        return false;
    }

    pub fn calculate_hash(&self, nonce: u128) -> String {
        hash::generate_hash(
            &self.reward,
            &self.data,
            self.timestamp,
            self.index,
            Some(&self.previous_hash),
            nonce,
        )
    }
}
