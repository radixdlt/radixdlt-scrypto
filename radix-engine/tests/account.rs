use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

fn fungible_amount() -> ResourceAmount {
    ResourceAmount::Fungible {
        amount: Decimal(100),
        resource_address: RADIX_TOKEN,
    }
}

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let other_key = executor.new_public_key();
    let other_account = executor.new_account(other_key);
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&fungible_amount(), account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();

    // Act
    let result = executor.run(transaction);

    // Assert
    assert!(result.unwrap().result.is_ok());
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let other_key = executor.new_public_key();
    let other_account = executor.new_account(other_key);
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&fungible_amount(), account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![other_key])
        .unwrap();

    // Act
    let result = executor.run(transaction);

    // Assert
    assert!(!result.unwrap().result.is_ok());
}

#[test]
fn account_to_bucket_to_account() {
    // Arrange
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let amount = fungible_amount();
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&amount, account)
        .take_from_worktop(amount.amount(), RADIX_TOKEN, |builder, bid| {
            builder.add_instruction(Instruction::CallMethod {
                component_address: account,
                method: "deposit".to_owned(),
                args: vec![scrypto_encode(&bid)],
            })
        })
        .build(vec![key])
        .unwrap();

    // Act
    let result = executor.run(transaction);

    // Assert
    assert!(result.unwrap().result.is_ok());
}
