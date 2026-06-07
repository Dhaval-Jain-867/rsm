use std::collections::HashMap;

use crate::transaction::Transaction;

#[derive(Clone)]
pub struct Balance {
    pub accounts: HashMap<[u8; 32], u64>
}

impl Balance {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new()
        }
    }

    pub fn set_balance(&mut self, name: [u8; 32], amount: u64) {
        self.accounts.insert(name, amount);
    }

    pub fn get_balance(&self, name: [u8; 32]) -> u64 {
        let balance = self.accounts.get(&name);
        match balance {
            Some(value) => *value,
            None => {
                return 0;
            }
        }
    }

    pub fn transfer(&mut self, transaction: &Transaction) {
        if let Some(balance) = self.accounts.get_mut(&transaction.payer) {
            *balance -= transaction.amount;
        }
        *self.accounts.entry(transaction.receiver).or_insert(0) += transaction.amount;
    }
}