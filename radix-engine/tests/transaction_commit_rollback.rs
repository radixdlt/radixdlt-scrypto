use radix_engine::{
    errors::{ApplicationError, RuntimeError},
    types::*,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_state_track_success() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10u32.into())
        .withdraw_from_account(account, RADIX_TOKEN)
        .call_method(
            other_account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let state_updates = &receipt.expect_commit().state_updates;
    for (o, n) in state_updates.down_substate_offsets() {
        println!("DOWN: {:?}, {}", o, n);
    }
    for (o, n) in state_updates.up_substate_offsets() {
        println!("UP: {:?}, {}", o, n);
    }
    assert_eq!(
        state_updates.down_substate_ids().len(),
        3 /* Global(Global) */
        + 2 /* AccessRulesChain(AccessRulesChain) */
        + 1 /* VaultAccessRulesChain(AccessRulesChain) */
        + 2 /* KeyValueStore(Entry([92, 130, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])) */
        + 2 /* Vault(Vault) */
        + 2 /* Account(Account) */
    );
    assert_eq!(
        state_updates.up_substate_ids().len(),
        3 /* Global(Global) */
        + 2 /* AccessRulesChain(AccessRulesChain) */
        + 1 /* VaultAccessRulesChain(AccessRulesChain) */
        + 2 /* KeyValueStore(Entry([92, 130, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])) */
        + 2 /* Vault(Vault) */
        + 2 /* Account(Account) */
    );
}

#[test]
fn test_state_track_failure() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10u32.into())
        .withdraw_from_account(account, RADIX_TOKEN)
        .call_method(
            other_account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .assert_worktop_contains_by_amount(Decimal::from(5u32), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
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
