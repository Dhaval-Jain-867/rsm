mod balances;
mod block;
mod transaction;

fn main() {
    println!("Hello, world!");
}

// #[test]
// fn init_blockchain() {
//     let mut new_blockchain = block::Blockchain::new();
//     assert_eq!(new_blockchain.is_valid(), true);

//     new_blockchain.add_block(String::from("This is genesis block"));
//     assert!(new_blockchain.is_valid());
//     new_blockchain.add_block(String::from("This is first block"));
//     assert!(new_blockchain.is_valid());
//     new_blockchain.add_block(String::from("This is second block"));

//     // let block_1 = & new_blockchain.chain[1];

//     new_blockchain.chain[1].data = String::from("I have changed data of 1st block");
//     assert!(!new_blockchain.is_valid());

//     // new_blockchain.chain[1].hash = block::generate_hash("I have changed data of 1st block", new_blockchain.chain[1].timestamp, new_blockchain.chain[1].index, Some(&new_blockchain.chain[1].previous_hash));
//     new_blockchain.chain[1].hash =
//         new_blockchain.chain[1].calculate_hash(new_blockchain.chain[1].nonce);
//     assert!(!new_blockchain.is_valid());
// }

// #[test]
// fn verifying_pow() {
//     let mut new_blockchain = block::Blockchain::new();
//     new_blockchain.add_block(String::from("This is genesis block"));
//     new_blockchain.add_block(String::from("This is first block"));
//     new_blockchain.add_block(String::from("This is second block"));

//     assert!(new_blockchain.chain[0].is_valid());
//     assert!(new_blockchain.chain[1].is_valid());
//     assert!(new_blockchain.chain[2].is_valid());

//     new_blockchain.chain[1].nonce += 1;
//     assert!(!new_blockchain.chain[1].is_valid());
//     new_blockchain.chain[1].mine();
//     assert!(new_blockchain.chain[1].is_valid());

//     new_blockchain.chain[1].hash = String::from("000abcdefghijklmnopqrstuvwxyz");
//     assert!(!new_blockchain.chain[1].is_valid());
//     new_blockchain.chain[1].mine();

//     new_blockchain.chain[1].data = String::from("I have changed data of 1st block");
//     new_blockchain.chain[1].mine();
//     assert!(!new_blockchain.is_valid());
// }

#[test]
fn init_transactions() {
    let mut new_blockchain = block::Blockchain::new();

    let pubkey_alice: [u8; 32] = [1; 32];
    let pubkey_bob: [u8; 32] = [2; 32];
    let pubkey_charlie: [u8; 32] = [3; 32];
    let pubkey_dave: [u8; 32] = [4; 32];

    let test_sign: [u8; 64] = [2; 64];
    let mut test_data = Vec::new();

    new_blockchain.mint(100, pubkey_alice);

    let trans_1 = transaction::Transaction {
        payer: pubkey_alice,
        reciever: pubkey_bob,
        amount: 60,
        signature: test_sign
    };
    test_data.push(trans_1);

    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&pubkey_alice], 40);
    assert_eq!(new_blockchain.balance.accounts[&pubkey_bob], 60);

    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&pubkey_alice], 40);
    assert_eq!(new_blockchain.balance.accounts[&pubkey_bob], 60);

    let trans_2 = transaction::Transaction {
        payer: pubkey_alice,
        reciever: pubkey_bob,
        amount: 30,
        signature: test_sign
    };
    test_data.clear();
    test_data.push(trans_2.clone());
    test_data.push(trans_2);
    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&pubkey_alice], 40);
    assert_eq!(new_blockchain.balance.accounts[&pubkey_bob], 60);

    test_data.clear();
    let trans_3 = transaction::Transaction {
        payer: pubkey_charlie,
        reciever: pubkey_bob,
        amount: 30,
        signature: test_sign
    };
    test_data.push(trans_3);
    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&pubkey_bob], 60);

    test_data.clear();
    let trans_4 = transaction::Transaction {
        payer: pubkey_alice,
        reciever: pubkey_dave,
        amount: 30,
        signature: test_sign
    };
    test_data.push(trans_4);
    new_blockchain.add_block(test_data.clone());

    assert_eq!(new_blockchain.balance.accounts[&pubkey_alice], 10);
    assert_eq!(new_blockchain.balance.accounts[&pubkey_bob], 60);
    assert_eq!(new_blockchain.balance.accounts[&pubkey_dave], 30);
}
