use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;
use std::fs;
use std::process::Command;

pub fn compile(name: &str) -> Vec<u8> {
    Command::new("cargo")
        .current_dir(format!("./tests/{}", name))
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();
    fs::read(format!(
        "./tests/{}/target/wasm32-unknown-unknown/release/{}.wasm",
        name,
        name.replace("-", "_")
    ))
    .unwrap()
}

fn fungible_amount() -> ResourceSpecification {
    ResourceSpecification::Fungible {
        amount: Decimal(100),
        resource_def_id: RADIX_TOKEN,
    }
}

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let other_key = executor.new_public_key();
    let other_account = executor.new_account(other_key);

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&fungible_amount(), account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let result = executor.run(transaction);

    // Assert
    assert!(result.unwrap().result.is_ok());
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("non_fungible")).unwrap();
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let other_key = executor.new_public_key();
    let other_account = executor.new_account(other_key);
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    let non_fungible_resource_def_id = receipt.new_resource_def_ids[0];
    let non_fungible_amount = ResourceSpecification::NonFungible {
        keys: BTreeSet::from([NonFungibleKey::from(1)]),
        resource_def_id: non_fungible_resource_def_id,
    };

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&non_fungible_amount, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let result = executor.run(transaction);

    // Assert
    println!("{:?}", result);
    assert!(result.unwrap().result.is_ok());
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let amount = fungible_amount();
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&amount, account)
        .take_from_worktop(&amount, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_id: account,
                    method: "deposit".to_owned(),
                    args: vec![scrypto_encode(&scrypto::resource::Bucket(bucket_id))],
                })
                .0
        })
        .build(vec![key])
        .unwrap();

    // Act
    let result = executor.run(transaction);

    // Assert
    assert!(result.unwrap().result.is_ok());
}
