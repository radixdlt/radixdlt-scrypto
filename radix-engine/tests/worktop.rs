use radix_engine::engine::KernelError;
use radix_engine::engine::RuntimeError;
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_worktop_resource_leak() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .withdraw_from_account(account, RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropNodeFailure(RENodeId::Worktop))
        )
    });
}
