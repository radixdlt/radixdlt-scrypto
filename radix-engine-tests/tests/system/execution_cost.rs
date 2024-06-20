#![cfg(feature = "std")]

use scrypto_test::prelude::*;
use std::path::PathBuf;

#[test]
fn transaction_previews_do_no_contains_debug_information() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_batch(account)
            .build(),
        vec![pk.into()],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: true,
        },
    );

    // Assert
    assert!(
        receipt.debug_information.is_none(),
        "Debug information is available in a preview receipt"
    );
}

#[test]
fn executing_transactions_with_debug_information_outputs_the_detailed_cost_breakdown() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest_with_execution_config(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_batch(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk)],
        ExecutionConfig::for_debug_transaction(),
    );

    // Assert
    assert!(
        receipt.debug_information.is_some(),
        "Debug information is not available when it should."
    );
}

#[test]
fn generate_flamegraph_of_faucet_free_method() -> Result<(), FlamegraphError> {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest_with_execution_config(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_batch(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk)],
        ExecutionConfig::for_debug_transaction(),
    );

    // Assert
    receipt.expect_commit_success();
    receipt.generate_execution_breakdown_flamegraph(
        PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("flamegraphs")
            .join("faucet-free-xrd.svg"),
        "Faucet Free XRD",
    )
}
