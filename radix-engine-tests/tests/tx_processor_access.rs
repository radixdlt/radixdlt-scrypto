use radix_engine::blueprints::resource::VaultError;
use radix_engine::errors::{
    ApplicationError, CallFrameError, KernelError, RuntimeError, SystemError,
};
use radix_engine::kernel::call_frame::{CreateNodeError, TakeNodeError, UnlockSubstateError};
use radix_engine::types::*;
use scrypto::prelude::FromPublicKey;
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn calling_transaction_processor_from_scrypto_should_not_panic() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/tx_processor_access");

    // Act
    let manifest_encoded_instructions: Vec<u8> = vec![0u8];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            package_address,
            "ExecuteManifest",
            "execute_manifest",
            manifest_args!(manifest_encoded_instructions),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(..))
        )
    });
}
