use sha2::{Sha256, Digest};

use crate::transaction::{CoinbaseTransaction, TransactionEnvelope};

pub fn generate_hash(coinbase: &CoinbaseTransaction, data: &Vec<TransactionEnvelope>, timestamp: i64, index: u128, previous_hash: Option<&str>, nonce: u128) -> String {
    let last_hash = previous_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let mut hasher = Sha256::new();
    let data_bytes = borsh::to_vec(data).expect("Failed to serialize transactions");
    hasher.update(data_bytes);
    hasher.update(borsh::to_vec(&coinbase).expect("Failed to serialize coinbase transaction"));
    hasher.update(timestamp.to_be_bytes());
    hasher.update(index.to_be_bytes());
    hasher.update(last_hash);
    hasher.update(nonce.to_be_bytes());
    let hash_result = hasher.finalize();
    let hex_hash = hex::encode(hash_result);
    return hex_hash;
}

pub fn is_hash_valid(hash: &str, difficulty: u8) -> bool {
    let mut test_str = String::from("");
    for _i in 0..difficulty {
        test_str.push('0');
    }
    let mut hash_str = String::from("");
    for i in 0..difficulty {
        hash_str.push(hash.chars().nth(i as usize).unwrap());  
    }
    if hash_str == test_str {
        return true;
    }
    return false;
}