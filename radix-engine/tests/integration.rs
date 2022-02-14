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
fn test_package() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("package")).unwrap();

    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "PackageTest",
            "publish_package",
            vec![],
            Some(account),
        )
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    assert!(receipt1.result.is_ok());
}

#[test]
fn test_context() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("context")).unwrap();

    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "ContextTest", "query", vec![], Some(account))
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    assert!(receipt1.result.is_ok());
}

#[test]
fn test_component() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("component")).unwrap();

    // Create component
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ComponentTest",
            "create_component",
            vec![],
            Some(account),
        )
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    assert!(receipt1.result.is_ok());

    // Find the component address from receipt
    let component = receipt1.new_component_refs[0];

    // Call functions & methods
    let transaction2 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            vec![component.to_string()],
            Some(account),
        )
        .call_method(component, "get_component_state", vec![], Some(account))
        .call_method(component, "put_component_state", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    assert!(receipt2.result.is_ok());
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

#[test]
fn test_bucket() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("bucket")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BucketTest", "combine", vec![], Some(account))
        .call_function(package, "BucketTest", "split", vec![], Some(account))
        .call_function(package, "BucketTest", "borrow", vec![], Some(account))
        .call_function(package, "BucketTest", "query", vec![], Some(account))
        .call_function(
            package,
            "BucketTest",
            "test_restricted_transfer",
            vec![],
            Some(account),
        )
        .call_function(package, "BucketTest", "test_burn", vec![], Some(account))
        .call_function(
            package,
            "BucketTest",
            "test_burn_freely",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_badge() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("badge")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BadgeTest", "combine", vec![], Some(account))
        .call_function(package, "BadgeTest", "split", vec![], Some(account))
        .call_function(package, "BadgeTest", "borrow", vec![], Some(account))
        .call_function(package, "BadgeTest", "query", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_call() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("call")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "MoveTest", "move_bucket", vec![], Some(account))
        .call_function(
            package,
            "MoveTest",
            "move_bucket_ref",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_non_fungible() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("non_fungible")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "take_and_put_bucket",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "take_and_put_vault",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "get_non_fungible_ids_bucket",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "get_non_fungible_ids_vault",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}
