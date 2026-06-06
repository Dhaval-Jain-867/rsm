use borsh;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use getrandom;
use hex;

use crate::{block::Blockchain, transaction::{Transaction, TransactionEnvelope}};

pub struct Wallet {
    pub public_key: [u8; 32],
    private_key: [u8; 32],
}

impl Wallet {
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

    pub fn sign_transaction(&self, payload: Transaction) -> TransactionEnvelope {
        let payload_bytes = borsh::to_vec(&payload).expect("Failed to serialize payload");
        let signing_key = SigningKey::from_bytes(&self.private_key);
        let signature_object = signing_key.sign(&payload_bytes);
        let signature_bytes = signature_object.to_bytes();

        TransactionEnvelope {
            payload: payload,
            signature: signature_bytes
        }
    }

    pub fn create_transaction(&self, to: [u8; 32], amount: u64) -> TransactionEnvelope {
        let payload = Transaction {
            payer: self.public_key,
            reciever: to,
            amount: amount
        };

        let transaction = self.sign_transaction(payload);
        return transaction;
    }
}
