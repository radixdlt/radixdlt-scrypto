use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_integer_basic_ops() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("math-ops-check");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function_with_abi(
            package_address,
            "Hello",
            "integer_basic_ops",
            vec!["55".to_string()],
            Some(account),
            &test_runner.export_abi(package_address, "Hello"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_native_and_safe_integer_interop() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("math-ops-check");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "Hello",
            "native_and_safe_integer_interop",
            args!(55u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_commit_success();
}
