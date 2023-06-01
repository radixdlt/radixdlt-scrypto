use radix_engine::blueprints::resource::NonFungibleResourceManagerError;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn mint_and_burn_of_non_fungible_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package,
            "MintAndBurn",
            "new",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success();
    let component_address = result.new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            component_address,
            "mint_and_burn",
            manifest_args!(1u64),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}