use std::fs;
use std::process::Command;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

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

#[test]
fn test_resource_def() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("resource_def")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible",
            vec![],
            Some(account),
        )
        .call_function(package, "ResourceTest", "query", vec![], Some(account))
        .call_function(package, "ResourceTest", "burn", vec![], Some(account))
        .call_function(
            package,
            "ResourceTest",
            "update_feature_flags",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "ResourceTest",
            "update_resource_metadata",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(!receipt.result.is_ok());

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "update_feature_flags_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(!receipt.result.is_ok());

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_resource_flags_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(!receipt.result.is_ok());

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_mutable_flags_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(!receipt.result.is_ok());

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_resource_permissions_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(!receipt.result.is_ok());
}
