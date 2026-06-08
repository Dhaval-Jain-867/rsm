use crate::block::Blockchain;

mod balances;
mod block;
mod hash;
mod miner;
mod transaction;
mod wallet;

fn main() {
    println!("Hello, world!");
}

// #[test]
// fn init_blockchain() {
//     let mut blockchain = block::Blockchain::new();
//     let miner = miner::Miner::new();

//     let wallet_a = wallet::Wallet::new();
//     let wallet_b = wallet::Wallet::new();

//     blockchain.mint(100, wallet_a.public_key);

//     let tx = wallet_a.create_transaction(wallet_b.public_key, 10);

//     blockchain.submit_transaction(tx).unwrap();

//     let (block, count) = miner.mine_block(&mut blockchain);
//     blockchain.add_block(block, count).unwrap();

//     assert!(blockchain.is_valid());

//     blockchain.chain[0].data[0].payload.amount = 50;

//     assert!(!blockchain.is_valid());

//     blockchain.chain[0].hash = blockchain.chain[0].calculate_hash(blockchain.chain[0].nonce);

//     assert!(!blockchain.is_valid());
// }

// #[test]
// fn verifying_pow() {
//     let mut blockchain = block::Blockchain::new();
//     let miner = miner::Miner::new();

//     let wallet_a = wallet::Wallet::new();
//     let wallet_b = wallet::Wallet::new();

//     blockchain.mint(100, wallet_a.public_key);

//     for _ in 0..3 {
//         let tx = wallet_a.create_transaction(wallet_b.public_key, 10);

//         blockchain.submit_transaction(tx).unwrap();

//         let (block, count) = miner.mine_block(&mut blockchain);
//         blockchain.add_block(block, count).unwrap();
//     }

//     assert!(blockchain.chain[0].is_valid());
//     assert!(blockchain.chain[1].is_valid());
//     assert!(blockchain.chain[2].is_valid());

//     blockchain.chain[1].nonce += 1;
//     assert!(!blockchain.chain[1].is_valid());

//     miner::Miner::hash_block(&mut blockchain.chain[1]);
//     assert!(blockchain.chain[1].is_valid());

//     blockchain.chain[1].hash = String::from("000abcdefghijklmnopqrstuvwxyz");

//     assert!(!blockchain.chain[1].is_valid());

//     miner::Miner::hash_block(&mut blockchain.chain[1]);

//     blockchain.chain[1].data[0].payload.amount = 999;

//     miner::Miner::hash_block(&mut blockchain.chain[1]);

//     assert!(!blockchain.is_valid());
// }

// #[test]
// fn init_transactions() {
//     let mut blockchain = block::Blockchain::new();
//     let miner = miner::Miner::new();

//     let wallet_alice = wallet::Wallet::new();
//     let wallet_bob = wallet::Wallet::new();
//     let wallet_charlie = wallet::Wallet::new();
//     let wallet_dave = wallet::Wallet::new();

//     blockchain.mint(100, wallet_alice.public_key);

//     // Alice -> Bob : 60
//     let trans_1 = transaction::Transaction {
//         payer: wallet_alice.public_key,
//         receiver: wallet_bob.public_key,
//         amount: 60,
//     };

//     let tx = wallet_alice.sign_transaction(trans_1);

//     blockchain.submit_transaction(tx).unwrap();

//     let (block, count) = miner.mine_block(&mut blockchain);
//     blockchain.add_block(block, count).unwrap();

//     assert_eq!(blockchain.balance.accounts[&wallet_alice.public_key], 40);
//     assert_eq!(blockchain.balance.accounts[&wallet_bob.public_key], 60);

//     // Alice tries to send another 60 (should fail)
//     let trans_2 = transaction::Transaction {
//         payer: wallet_alice.public_key,
//         receiver: wallet_bob.public_key,
//         amount: 60,
//     };

//     let tx = wallet_alice.sign_transaction(trans_2);

//     assert!(blockchain.submit_transaction(tx).is_err());

//     assert_eq!(blockchain.balance.accounts[&wallet_alice.public_key], 40);
//     assert_eq!(blockchain.balance.accounts[&wallet_bob.public_key], 60);

//     // Double spend inside same block
//     let trans_3 = transaction::Transaction {
//         payer: wallet_alice.public_key,
//         receiver: wallet_bob.public_key,
//         amount: 30,
//     };

//     let tx1 = wallet_alice.sign_transaction(trans_3.clone());
//     let tx2 = wallet_alice.sign_transaction(trans_3);

//     blockchain.submit_transaction(tx1).unwrap();
//     blockchain.submit_transaction(tx2).unwrap();

//     let (block, count) = miner.mine_block(&mut blockchain);
//     blockchain.add_block(block, count).unwrap();

//     assert_eq!(blockchain.balance.accounts[&wallet_alice.public_key], 10);
//     assert_eq!(blockchain.balance.accounts[&wallet_bob.public_key], 90);

//     // Charlie has no balance
//     let trans_4 = transaction::Transaction {
//         payer: wallet_charlie.public_key,
//         receiver: wallet_bob.public_key,
//         amount: 30,
//     };

//     let tx = wallet_charlie.sign_transaction(trans_4);

//     assert!(blockchain.submit_transaction(tx).is_err());

//     assert_eq!(blockchain.balance.accounts[&wallet_bob.public_key], 90);

//     // Alice now has only 10, tries to send 30
//     let trans_5 = transaction::Transaction {
//         payer: wallet_alice.public_key,
//         receiver: wallet_dave.public_key,
//         amount: 30,
//     };

//     let tx = wallet_alice.sign_transaction(trans_5);

//     assert!(blockchain.submit_transaction(tx).is_err());

//     assert_eq!(blockchain.balance.accounts[&wallet_alice.public_key], 10);
//     assert_eq!(blockchain.balance.accounts[&wallet_bob.public_key], 90);
//     assert!(!blockchain.balance.accounts.contains_key(&wallet_dave.public_key));
// }

// #[test]
// fn init_signatures() {
//     let mut new_blockchain = block::Blockchain::new();

//     let wallet_alice = wallet::Wallet::new();
//     let wallet_bob = wallet::Wallet::new();
//     let wallet_charlie = wallet::Wallet::new();
//     let wallet_dave = wallet::Wallet::new();

//     let mut test_data = Vec::new();

//     new_blockchain.mint(100, wallet_alice.public_key);

//     let trans_1 = transaction::Transaction {
//         payer: wallet_alice.public_key,
//         receiver: wallet_bob.public_key,
//         amount: 60,
//     };
//     test_data.push(wallet_alice.sign_transaction(trans_1));
//     assert!(test_data[0].verify_signature());

//     test_data[0].payload.amount = 50;
//     assert!(!test_data[0].verify_signature());

//     test_data[0].payload.amount = 60;
//     test_data[0].payload.receiver = [1; 32];
//     assert!(!test_data[0].verify_signature());

//     test_data[0].payload.receiver = wallet_bob.public_key;
//     test_data[0].payload.payer = wallet_charlie.public_key;
//     assert!(!test_data[0].verify_signature());

//     test_data[0].payload.payer = wallet_alice.public_key;
//     test_data[0].signature = [1; 64];
//     assert!(
//         new_blockchain
//             .submit_transaction(test_data[0].clone())
//             .is_err()
//     );
// }

// #[test]
// fn init_miner() {
//     let mut new_blockchain = block::Blockchain::new();

//     let wallet_alice = wallet::Wallet::new();
//     let wallet_bob = wallet::Wallet::new();

//     let miner_crazy = miner::Miner::new();

//     new_blockchain.mint(100, wallet_alice.public_key);

//     // creating a transaction & adding it to mempool
//     let tx1 = wallet_alice.create_transaction(wallet_bob.public_key, 10);
//     new_blockchain.submit_transaction(tx1.clone());
//     assert_eq!(new_blockchain.mempool.len(), 1); // checking if mempool was updated

//     // adding 4 transactions
//     new_blockchain.submit_transaction(tx1.clone());
//     new_blockchain.submit_transaction(tx1.clone());
//     new_blockchain.submit_transaction(tx1);
//     assert_eq!(new_blockchain.mempool.len(), 4);

//     let (new_block, to_rem) = miner_crazy.mine_block(&mut new_blockchain);
//     new_blockchain.add_block(new_block, to_rem);

//     assert_eq!(new_blockchain.chain.len(), 1);
//     assert!(new_blockchain.chain[0].is_valid());
//     assert_eq!(new_blockchain.mempool.len(), 1);
// }

// #[test]
// fn mining_rewards() {
//     let mut new_blockchain = block::Blockchain::new();

//     let wallet_alice = wallet::Wallet::new();
//     let wallet_bob = wallet::Wallet::new();

//     let miner_crazy = miner::Miner::new();

//     new_blockchain.mint(100, wallet_alice.public_key);
//     let tx1 = wallet_alice.create_transaction(wallet_bob.public_key, 10);
//     new_blockchain.submit_transaction(tx1.clone());
//     new_blockchain.submit_transaction(tx1.clone());
//     new_blockchain.submit_transaction(tx1.clone());
//     new_blockchain.submit_transaction(tx1);

//     let (new_block, to_rem) = miner_crazy.mine_block(&mut new_blockchain);
//     new_blockchain.add_block(new_block, to_rem);

//     assert_eq!(new_blockchain.balance.accounts[&miner_crazy.public_key], 50);

//      let (new_block, to_rem) = miner_crazy.mine_block(&mut new_blockchain);
//     new_blockchain.add_block(new_block, to_rem);

//     assert_eq!(new_blockchain.balance.accounts[&miner_crazy.public_key], 100);
// }

#[test]
fn rebuild_state_test() {
    let (mut new_blockchain, genesis_wallet) =
        block::Blockchain::new(1000).unwrap();

    let wallet_bob = wallet::Wallet::new();
    let wallet_charlie = wallet::Wallet::new();
    let wallet_dave = wallet::Wallet::new();

    let miner_xod = miner::Miner::new();

    let tx1 = genesis_wallet.create_transaction(
        wallet_bob.public_key,
        200,
    );

    new_blockchain.submit_transaction(tx1).unwrap();

    let (block1, count1) =
        miner_xod.mine_block(&mut new_blockchain).unwrap();

    new_blockchain.add_block(block1, count1).unwrap();

    let tx2 = wallet_bob.create_transaction(
        wallet_charlie.public_key,
        50,
    );

    new_blockchain.submit_transaction(tx2).unwrap();

    let (block2, count2) =
        miner_xod.mine_block(&mut new_blockchain).unwrap();

    new_blockchain.add_block(block2, count2).unwrap();

    let tx3 = wallet_charlie.create_transaction(
        wallet_dave.public_key,
        20,
    );

    new_blockchain.submit_transaction(tx3).unwrap();

    let (block3, count3) =
        miner_xod.mine_block(&mut new_blockchain).unwrap();

    new_blockchain.add_block(block3, count3).unwrap();

    // Rebuild state from chain
    let rebuilt_state =
        new_blockchain.rebuild_state().unwrap();

    assert_eq!(
        rebuilt_state.accounts,
        new_blockchain.balance.accounts
    );

    new_blockchain.chain[1].data[0].payload.amount += 100;

    assert!(new_blockchain.rebuild_state().is_err());
}
