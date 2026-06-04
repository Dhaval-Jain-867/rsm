use borsh::BorshSerialize;

use crate::balances::Balance;

#[derive(BorshSerialize)]
pub struct Transaction {
    payer: [u8; 32],
    reciever: [u8; 32],
    amount: u64,
    signature: [u8; 64]
}

impl Transaction {
    pub fn is_valid(&self, balances_instance: &Balance) -> bool {
        
    }
}