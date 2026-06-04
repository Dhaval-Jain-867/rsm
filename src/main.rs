mod balances;
mod block;
mod transaction;

fn main() {
    println!("Hello, world!");
}

#[test]
fn init_blockchain() {
    let mut new_blockchain = block::Blockchain::new();
    assert_eq!(new_blockchain.is_valid(), true);

    new_blockchain.add_block(String::from("This is genesis block"));
    assert!(new_blockchain.is_valid());
    new_blockchain.add_block(String::from("This is first block"));
    assert!(new_blockchain.is_valid());
    new_blockchain.add_block(String::from("This is second block"));

    // let block_1 = & new_blockchain.chain[1];

    new_blockchain.chain[1].data = String::from("I have changed data of 1st block");
    assert!(!new_blockchain.is_valid());
    
    // new_blockchain.chain[1].hash = block::generate_hash("I have changed data of 1st block", new_blockchain.chain[1].timestamp, new_blockchain.chain[1].index, Some(&new_blockchain.chain[1].previous_hash));
    new_blockchain.chain[1].hash = new_blockchain.chain[1].calculate_hash(new_blockchain.chain[1].nonce);
    assert!(!new_blockchain.is_valid());
}

#[test]
fn verifying_pow() {
    let mut new_blockchain = block::Blockchain::new();
    new_blockchain.add_block(String::from("This is genesis block"));
    new_blockchain.add_block(String::from("This is first block"));
    new_blockchain.add_block(String::from("This is second block"));

    assert!(new_blockchain.chain[0].is_valid());
    assert!(new_blockchain.chain[1].is_valid());
    assert!(new_blockchain.chain[2].is_valid());

    new_blockchain.chain[1].nonce += 1;
    assert!(!new_blockchain.chain[1].is_valid());
    new_blockchain.chain[1].mine();
    assert!(new_blockchain.chain[1].is_valid());

    new_blockchain.chain[1].hash = String::from("000abcdefghijklmnopqrstuvwxyz");
    assert!(!new_blockchain.chain[1].is_valid());
    new_blockchain.chain[1].mine();

    new_blockchain.chain[1].data = String::from("I have changed data of 1st block");
    new_blockchain.chain[1].mine();
    assert!(!new_blockchain.is_valid());
}

#[test]
fn init_balances() {
    let mut my_balance = balances::Pallet::new();

    assert_eq!(my_balance.get_balance(&String::from("alice")), 0);
    my_balance.set_balance(String::from("alice"), 100);
    assert_eq!(my_balance.get_balance(&String::from("alice")), 100);
    assert_eq!(my_balance.get_balance(&String::from("bob")), 0);
}