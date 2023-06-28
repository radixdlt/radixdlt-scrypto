use radix_engine::errors::ApplicationError;
use radix_engine::errors::RuntimeError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_queries::typed_substate_layout::VaultError;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_lock_fee_and_then_withdraw_failure() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .withdraw_from_account(account, RADIX_TOKEN, dec!("1000000"))
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
                VaultError::ResourceError(ResourceError::InsufficientBalance)
            ))
        )
    });
}
