use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn deposit_test() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let pub_key = executor.new_public_key();
    let account = executor.create_account(pub_key);
    let package = executor.publish_package(include_code!());

    // we can't use ResourceBuilder in tests, so this is a workaround to create resources
    let token_b_address = create_token(
        &mut executor, package, account, pub_key, "Token B".to_owned(), "tokenB".to_owned()
    );

    let token_c_address: Address = create_token(
        &mut executor, package, account, pub_key, "Token C".to_owned(), "tokenC".to_owned()
    );

    let auto_lend_address = create_auto_lend(
        &mut executor, package, token_b_address, token_c_address, pub_key
    );

    // before depositing, liquidity should be zero
    assert_eq!(b_tokens_liquidity(&mut executor, auto_lend_address, pub_key), Amount::from(0));

    // before depositing, LP supply should be equal to 0
    assert_eq!(a_b_tokens_supply(&mut executor, auto_lend_address, pub_key), Amount::from(0));

    deposit_token_b(
        &mut executor, auto_lend_address, token_b_address, account, Amount::from(100), pub_key
    );
    
    // after depositing, liquidity should be equal to amount deposited
    assert_eq!(b_tokens_liquidity(&mut executor, auto_lend_address, pub_key), Amount::from(100));

    // after depositing, LP supply should be equal to 1
    assert_eq!(a_b_tokens_supply(&mut executor, auto_lend_address, pub_key), Amount::from(100));
}

#[test]
fn two_deposits_test() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let pub_key = executor.new_public_key();
    let account = executor.create_account(pub_key);
    let package = executor.publish_package(include_code!());

    // we can't use ResourceBuilder in tests, so this is a workaround to create resources
    let token_b_address = create_token(
        &mut executor, package, account, pub_key, "Token B".to_owned(), "tokenB".to_owned()
    );

    let token_c_address: Address = create_token(
        &mut executor, package, account, pub_key, "Token C".to_owned(), "tokenC".to_owned()
    );

    let auto_lend_address = create_auto_lend(
        &mut executor, package, token_b_address, token_c_address, pub_key
    );

    // before depositing, liquidity should be zero
    assert_eq!(b_tokens_liquidity(&mut executor, auto_lend_address, pub_key), Amount::from(0));

    // before depositing, LP supply should be equal to 0
    assert_eq!(a_b_tokens_supply(&mut executor, auto_lend_address, pub_key), Amount::from(0));

    deposit_token_b(
        &mut executor, auto_lend_address, token_b_address, account, Amount::from(100), pub_key
    );

    // after first deposit, liquidity should be equal to 100
    assert_eq!(b_tokens_liquidity(&mut executor, auto_lend_address, pub_key), Amount::from(100));

    // after depositing, LP supply should be equal to 100
    assert_eq!(a_b_tokens_supply(&mut executor, auto_lend_address, pub_key), Amount::from(100));

    deposit_token_b(
        &mut executor, auto_lend_address, token_b_address, account, Amount::from(10), pub_key
    );
    
    // after second depositing, liquidity should be equal to 110
    assert_eq!(b_tokens_liquidity(&mut executor, auto_lend_address, pub_key), Amount::from(110));

    // after depositing, LP supply should be equal to 110
    assert_eq!(a_b_tokens_supply(&mut executor, auto_lend_address, pub_key), Amount::from(110));
}

fn create_token(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    package: Address,
    account: Address,
    signer: Address,
    name: String, 
    symbol: String
) -> Address {
    let create_token_c_tx = TransactionBuilder::new(executor)
        .call_function(package, "Token", "new", vec![name, symbol], Some(account))
        .deposit_all(account)
        .build(vec![signer])
        .unwrap();
    let create_token_c_tx_receipt = executor.run(create_token_c_tx, false).unwrap();
    let token_c_address: Address = create_token_c_tx_receipt.resource_def(0).unwrap();
    return token_c_address;
}

fn create_auto_lend(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    package: Address,
    token_b_address: Address,
    token_c_address: Address,
    signer: Address
) -> Address {
    let create_auto_lend_tx = TransactionBuilder::new(executor)
        .call_function(
            package, 
            "AutoLend", 
            "new", 
            vec![token_b_address.to_string(), token_c_address.to_string()], None
        )
        .build(vec![signer])
        .unwrap();
    let create_auto_lend_tx_receipt = executor.run(create_auto_lend_tx, false).unwrap();
    println!("{:?}\n", create_auto_lend_tx_receipt);
    let auto_lend_address = create_auto_lend_tx_receipt.component(0).unwrap();
    return auto_lend_address;
}

fn deposit_token_b(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    auto_lend_address: Address,
    token_b_address: Address,
    account: Address,
    amount: Amount,
    signer: Address
) {
    let tx = TransactionBuilder::new(executor)
        .call_method(auto_lend_address, "deposit", vec![format!("{},{}", amount, token_b_address), ], Some(account))
        .deposit_all(account)
        .build(vec![signer])
        .unwrap();
    let receipt = executor.run(tx, false).unwrap();
    println!("{:?}\n", receipt);
}

fn a_b_tokens_supply(
    executor: &mut TransactionExecutor<InMemoryLedger>, 
    auto_lend_address: Address, 
    signer: Address
) -> Amount {
    let tx = TransactionBuilder::new(executor)
    .call_method(auto_lend_address, "a_b_tokens_supply", vec![], None)
    .build(vec![signer])
    .unwrap();

    let receipt3 = executor.run(tx, false).unwrap();
    return scrypto_decode(&receipt3.results[0].as_ref().unwrap().as_ref().unwrap().encoded).unwrap();
}

fn b_tokens_liquidity(
    executor: &mut TransactionExecutor<InMemoryLedger>, 
    auto_lend_address: Address, signer: Address
) -> Amount {
    let tx = TransactionBuilder::new(executor)
    .call_method(auto_lend_address, "b_tokens_liquidity", vec![], None)
    .build(vec![signer])
    .unwrap();

    let receipt3 = executor.run(tx, false).unwrap();
    return scrypto_decode(&receipt3.results[0].as_ref().unwrap().as_ref().unwrap().encoded).unwrap();
}
