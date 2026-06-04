mod balances;
mod block;
mod transaction;
mod wallet;

fn main() {
    println!("Hello, world!");
}

#[test]
fn init_blockchain() {
    let mut new_blockchain = block::Blockchain::new();

    let wallet_a = wallet::Wallet::new();
    let wallet_b = wallet::Wallet::new();

    new_blockchain.mint(100, wallet_a.public_key);

    let tx = transaction::Transaction {
        payer: wallet_a.public_key,
        reciever: wallet_b.public_key,
        amount: 10,
    };

    let envelope = wallet_a.sign_transaction(tx);

    assert!(new_blockchain.is_valid());

    new_blockchain.add_block(vec![envelope.clone()]);
    assert!(new_blockchain.is_valid());

    new_blockchain.add_block(vec![envelope.clone()]);
    assert!(new_blockchain.is_valid());

    new_blockchain.add_block(vec![envelope.clone()]);

    new_blockchain.chain[1].data[0].payload.amount = 50;
    assert!(!new_blockchain.is_valid());

    new_blockchain.chain[1].hash =
        new_blockchain.chain[1].calculate_hash(new_blockchain.chain[1].nonce);

    assert!(!new_blockchain.is_valid());
}

#[test]
fn verifying_pow() {
    let mut new_blockchain = block::Blockchain::new();

    let wallet_a = wallet::Wallet::new();
    let wallet_b = wallet::Wallet::new();

    new_blockchain.mint(100, wallet_a.public_key);

    let tx = transaction::Transaction {
        payer: wallet_a.public_key,
        reciever: wallet_b.public_key,
        amount: 10,
    };

    let envelope = wallet_a.sign_transaction(tx);

    new_blockchain.add_block(vec![envelope.clone()]);
    new_blockchain.add_block(vec![envelope.clone()]);
    new_blockchain.add_block(vec![envelope.clone()]);

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

    new_blockchain.chain[1].data[0].payload.amount = 999;

    new_blockchain.chain[1].mine();

    assert!(!new_blockchain.is_valid());
}

#[test]
fn init_transactions() {
    let mut new_blockchain = block::Blockchain::new();

    let wallet_alice = wallet::Wallet::new();
    let wallet_bob = wallet::Wallet::new();
    let wallet_charlie = wallet::Wallet::new();
    let wallet_dave = wallet::Wallet::new();

    let mut test_data = Vec::new();

    new_blockchain.mint(100, wallet_alice.public_key);

    let trans_1 = transaction::Transaction {
        payer: wallet_alice.public_key,
        reciever: wallet_bob.public_key,
        amount: 60,
    };
    test_data.push(wallet_alice.sign_transaction(trans_1));

    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&wallet_alice.public_key], 40);
    assert_eq!(new_blockchain.balance.accounts[&wallet_bob.public_key], 60);

    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&wallet_alice.public_key], 40);
    assert_eq!(new_blockchain.balance.accounts[&wallet_bob.public_key], 60);

    let trans_2 = transaction::Transaction {
        payer: wallet_alice.public_key,
        reciever: wallet_bob.public_key,
        amount: 30,
    };
    test_data.clear();
    test_data.push(wallet_alice.sign_transaction(trans_2.clone()));
    test_data.push(wallet_alice.sign_transaction(trans_2));
    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&wallet_alice.public_key], 40);
    assert_eq!(new_blockchain.balance.accounts[&wallet_bob.public_key], 60);

    test_data.clear();
    let trans_3 = transaction::Transaction {
        payer: wallet_charlie.public_key,
        reciever: wallet_bob.public_key,
        amount: 30,
    };
    test_data.push(wallet_charlie.sign_transaction(trans_3));
    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&wallet_bob.public_key], 60);

    test_data.clear();
    let trans_4 = transaction::Transaction {
        payer: wallet_alice.public_key,
        reciever: wallet_dave.public_key,
        amount: 30,
    };
    test_data.push(wallet_alice.sign_transaction(trans_4));
    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&wallet_alice.public_key], 10);
    assert_eq!(new_blockchain.balance.accounts[&wallet_bob.public_key], 60);
    assert_eq!(new_blockchain.balance.accounts[&wallet_dave.public_key], 30);
}
