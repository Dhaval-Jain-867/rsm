use std::collections::HashMap;

use borsh::BorshSerialize;

use crate::balances::Balance;

#[derive(BorshSerialize, Clone)]
pub struct Transaction {
    pub payer: [u8; 32],
    pub reciever: [u8; 32],
    pub amount: u64,
    pub signature: [u8; 64]
}

impl Transaction {
    pub fn is_valid(&self, balances: &Balance) -> bool {
        balances.accounts.contains_key(&self.payer) && balances.accounts[&self.payer] >= self.amount && self.amount > 0
    }
}