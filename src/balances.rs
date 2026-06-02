use std::collections::BTreeMap;

pub struct Pallet {
    pub balances: BTreeMap<String, u128>
}

impl Pallet {
    pub fn new() -> Self {
        Self {
            balances: BTreeMap::new()
        }
    }

    pub fn set_balance(&mut self, name: String, amount: u128) {
        self.balances.insert(name, amount);
    }

    pub fn get_balance(&self, name: &String) -> u128 {
        let balance = self.balances.get(name);
        match balance {
            Some(value) => *value,
            None => {
                return 0;
            }
        }
    }
}