use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use utils::ContextualDisplay;

#[test]
fn test_integer_basic_ops() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, _) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/math-ops-check");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Hello",
            "integer_basic_ops",
            args!(String::from("55")),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_native_and_safe_integer_interop() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/math-ops-check");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Hello",
            "native_and_safe_integer_interop",
            args!(55u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}
