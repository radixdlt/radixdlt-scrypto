#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn self_transfer_txn(account: ComponentAddress, amount: Decimal) -> TransactionManifest {
    ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account_by_amount(amount, RADIX_TOKEN, account)
        .call_method_with_all_resources(account, "deposit_batch")
        .build()
}

#[test]
fn batched_execution_should_match_one_by_one_execution() {
    // Arrange
    // These two runners should mirror each other
    let mut store0 = InMemorySubstateStore::with_bootstrap();
    let mut test_runner0 = TestRunner::new(true, &mut store0);
    let mut store1 = InMemorySubstateStore::with_bootstrap();
    let mut test_runner1 = TestRunner::new(true, &mut store1);
    let (public_key, _, account) = test_runner0.new_account();
    let _ = test_runner1.new_account();
    let mut manifests = Vec::new();
    for amount in 0..10 {
        let manifest = self_transfer_txn(account, Decimal::from(amount));
        manifests.push((manifest, vec![public_key]));
    }

    // Act
    for (manifest, signers) in &manifests {
        let receipt = test_runner0.execute_manifest(manifest.clone(), signers.clone());
        receipt.expect_success();
    }
    test_runner1.execute_batch(manifests);

    // Assert
    assert_eq!(store0, store1);
}
