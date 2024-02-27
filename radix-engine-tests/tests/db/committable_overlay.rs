use scrypto::prelude::*;
use scrypto_test::ledger_simulator::*;
use substate_store_impls::committable_overlay::*;
use substate_store_impls::memory_db::*;
use transaction::builder::*;

// We're testing to make sure that any changes that we write to the overlay can be read later on
// and that the engine can work over them. For this test we do something very simple. Create two
// accounts and perform a transfer between them. If all of these transactions succeed then the
// accounts were written to the overlay successfully and all subsequent state changes were also
// written successfully.
#[test]
fn changes_written_to_overlay_are_accessible_later() {
    // Arrange
    let database = CommittableOverlay::new(InMemorySubstateDatabase::standard());
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_database(database)
        .without_kernel_trace()
        .build();

    let (public_key1, _, account1) = ledger.new_account(false);
    let (public_key2, _, account2) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account1, XRD, dec!(100))
            .deposit_batch(account2)
            .build(),
        [public_key1, public_key2]
            .map(|pk| NonFungibleGlobalId::from_public_key(&pk))
            .to_vec(),
    );

    // Assert
    receipt.expect_commit_success();
}
