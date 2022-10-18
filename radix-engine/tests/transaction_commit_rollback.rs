use radix_engine::engine::*;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_state_track_success() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10u32.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method(
            other_account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        14,
        receipt.expect_commit().state_updates.down_substates.len()
    );
    assert_eq!(14, receipt.expect_commit().state_updates.up_substates.len());
}

#[test]
fn test_state_track_failure() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10u32.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method(
            other_account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .assert_worktop_contains_by_amount(Decimal::from(5u32), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::WorktopError(_))
        )
    });
    assert_eq!(
        1,
        receipt.expect_commit().state_updates.down_substates.len()
    ); // only the vault is down
    assert_eq!(1, receipt.expect_commit().state_updates.up_substates.len());
}
