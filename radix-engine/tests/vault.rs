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
fn dangling_vault_should_not_be_allowed() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let key = sut.new_public_key();
    let account = sut.new_account(key);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "dangling_vault", vec![], Some(account))
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}