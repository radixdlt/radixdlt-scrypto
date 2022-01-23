use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn withdrawing_from_account_i_own_should_succeed() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let other_key = executor.new_public_key();
    let other_account = executor.new_account(other_key);
    let resource = ResourceAmount::Fungible {
        amount: Decimal(100),
        resource_address: RADIX_TOKEN
    };
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&resource, account)
        .deposit_all_buckets(other_account)
        .build(vec![key]).unwrap();
    let result = executor.run(transaction, false);
    assert!(result.unwrap().success);
}

#[test]
fn withdrawing_from_account_i_dont_own_should_fail() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let other_key = executor.new_public_key();
    let other_account = executor.new_account(other_key);
    let resource = ResourceAmount::Fungible {
        amount: Decimal(100),
        resource_address: RADIX_TOKEN
    };
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&resource, account)
        .deposit_all_buckets(other_account)
        .build(vec![other_key]).unwrap();
    let result = executor.run(transaction, false);
    assert!(!result.unwrap().success);
}