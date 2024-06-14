#![cfg(feature = "std")]

use scrypto_test::prelude::*;
use std::path::PathBuf;

#[test]
fn generate_flamegraph_of_faucet_free_method() -> Result<(), FlamegraphError> {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_batch(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk)],
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
