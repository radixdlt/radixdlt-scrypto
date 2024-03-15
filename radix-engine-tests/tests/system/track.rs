use radix_common::prelude::*;
use radix_engine::errors::ApplicationError;
use radix_engine::errors::RuntimeError;
use radix_engine_interface::types::FromPublicKey;
use radix_substate_store_queries::typed_substate_layout::VaultError;
use scrypto_test::prelude::*;

#[test]
fn test_lock_fee_and_then_withdraw_failure() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .withdraw_from_account(account, XRD, dec!("1000000"))
        .build();
    let receipt = ledger.execute_manifest(
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
