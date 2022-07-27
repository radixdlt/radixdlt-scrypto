#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_state_track_success() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
    assert!(
        !receipt.state_updates.down_substates.is_empty()
            && !receipt.state_updates.up_substates.is_empty()
    )
}

#[test]
fn test_state_track_failure() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .assert_worktop_contains_by_amount(Decimal::from(5), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::WorktopError(_)));
    assert!(
        receipt.state_updates.down_substates.is_empty()
            && receipt.state_updates.up_substates.is_empty()
    )
}
