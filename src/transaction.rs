use borsh::{BorshSerialize, to_vec};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::balances::Balance;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};

#[derive(BorshSerialize, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub payer: [u8; 32],
    pub receiver: [u8; 32],
    pub amount: u64,
    pub fees: u64,
}

#[derive(BorshSerialize, Clone, Serialize, Deserialize)]
pub struct TransactionEnvelope {
    pub payload: Transaction,
    #[serde(with = "BigArray")]
    pub signature: [u8; 64]
}

#[derive(BorshSerialize, Clone, Serialize, Deserialize)]
pub struct CoinbaseTransaction {
    pub receiver: [u8; 32],
    pub amount: u64
}

impl Transaction {
    pub fn is_valid(&self, balances: &Balance) -> bool {
        balances.accounts.contains_key(&self.payer) && balances.accounts[&self.payer] >= self.amount.checked_add(self.fees).unwrap() && self.amount > 0
    }
}

impl TransactionEnvelope {
    pub fn is_valid(&self, balances: &Balance) -> bool {
        self.verify_signature() && self.payload.is_valid(balances)
    }

    pub fn verify_signature(&self) -> bool {
        let public_key = match VerifyingKey::from_bytes(&self.payload.payer) {
            Ok(key) => key,
            Err(_) => return false
        };

        let signature = Signature::from_bytes(&self.signature);

        let payload_bytes = match to_vec(&self.payload) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        public_key.verify(&payload_bytes, &signature).is_ok()
    }
}