use std::collections::HashMap;

pub struct Balance {
    pub balances: HashMap<[u8; 32], u64>
}

impl Balance {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new()
        }
    }

    pub fn set_balance(&mut self, name: [u8; 32], amount: u64) {
        self.balances.insert(name, amount);
    }

    pub fn get_balance(&self, name: [u8; 32]) -> u64 {
        let balance = self.balances.get(&name);
        match balance {
            Some(value) => *value,
            None => {
                return 0;
            }
        }
    }
}