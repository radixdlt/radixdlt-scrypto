use radix_engine::errors::ApplicationError;
use radix_engine::errors::RuntimeError;
use radix_engine_common::prelude::*;
use radix_engine_interface::types::FromPublicKey; 
use substate_store_queries::typed_substate_layout::VaultError;
use scrypto_test::prelude::*;

#[test]
fn test_lock_fee_and_then_withdraw_failure() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .withdraw_from_account(account, XRD, dec!("1000000"))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::ResourceError(ResourceError::InsufficientBalance { .. })
            ))
        )
    });
}
