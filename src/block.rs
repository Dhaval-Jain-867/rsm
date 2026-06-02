use chrono::Utc;
use sha2::{Sha256, Digest};
use hex;

fn generate_hash(data: &str, timestamp: i64, index: u128, previous_hash: Option<&str>, nonce: u128) -> String {
    let last_hash = previous_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.update(timestamp.to_be_bytes());
    hasher.update(index.to_be_bytes());
    hasher.update(last_hash);
    hasher.update(nonce.to_be_bytes());
    let hash_result = hasher.finalize();
    let hex_hash = hex::encode(hash_result);
    return hex_hash;
}

fn is_hash_valid(hash: &str, difficulty: u8) -> bool {
    let mut test_str = String::from("");
    for _i in 0..difficulty {
        test_str.push('0');
    }
    let mut hash_str = String::from("");
    for i in 0..difficulty {
        hash_str.chars().nth(i as usize);
    }
    if hash_str == test_str {
        return true;
    }
    return false;
}

pub struct Block {
    pub index: u128,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u128,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            chain: Vec::new()
        }
    }

    pub fn add_block(&mut self, data: String) {
        let last_block = self.chain.last();
        let timestamp = Utc::now().timestamp();

        match last_block {
            Some(l_block) => {
                // let hash = generate_hash(&data, timestamp, l_block.index + 1, Some(&l_block.hash));
                let mut new_block = Block::new(l_block.index + 1, timestamp, data, l_block.hash.clone());
                // let mut new_block = Block {
                //     index: l_block.index + 1,
                //     timestamp: timestamp,
                //     data: data,
                //     previous_hash: l_block.hash.clone(),
                //     hash: String::from(""),
                //     nonce: 0
                // };
                new_block.mine();
                self.chain.push(new_block);
            }
            None => {
                // let hash = generate_hash(&data, timestamp, 0, None);
                let mut new_block = Block::new(0, timestamp, data, String::from("0000000000000000000000000000000000000000000000000000000000000000"));
                let mut new_block = Block {
                    index: 0,
                    timestamp: timestamp,
                    data: data,
                    previous_hash: String::from("0000000000000000000000000000000000000000000000000000000000000000"),
                    hash: String::from(""),
                    nonce: 0
                };
                new_block.mine();
                self.chain.push(new_block);
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        let chain_length = self.chain.len();
        for i in 0..chain_length {
            let curr_block = &self.chain[i];
            if i != 0 {
                if curr_block.previous_hash == (&self.chain[i-1]).hash && curr_block.is_valid() {
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
}

impl Block {
    pub fn new(index: u128, timestamp: i64, data: String, previous_hash: String) -> Self {
        Self {
            index: index,
            timestamp: timestamp,
            data: data,
            previous_hash: previous_hash,
            hash: String::from(""),
            nonce: 0
        }
    }
    pub fn is_valid(&self) -> bool {
        if self.hash == self.calculate_hash(self.nonce) && is_hash_valid(&self.hash, 3) {
            return true;
        }
        return false;
    }

    pub fn calculate_hash(&self, nonce: u128) -> String {
        generate_hash(&self.data, self.timestamp, self.index, Some(&self.previous_hash), nonce)
    }

    pub fn mine(&mut self) {
        let mut nonce: u128 = 0;
        loop {
            let our_hash = self.calculate_hash(nonce);
            if is_hash_valid(&our_hash, 3) {
                self.hash = our_hash;
                self.nonce = nonce;
                break;
            }
            nonce += 1;
        }
    }
}