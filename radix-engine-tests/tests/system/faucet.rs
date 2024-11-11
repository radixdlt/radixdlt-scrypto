use radix_common::prelude::*;
use radix_engine::updates::BabylonSettings;
use scrypto_test::prelude::*;

#[test]
fn lock_fee_on_empty_faucet_should_give_nice_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| {
                    BabylonSettings::test_default().with_faucet_supply(Decimal::ZERO)
                })
                .from_bootstrap_to_latest()
        })
        .build();

    // Act
    let manifest = ManifestBuilder::new().lock_fee_from_faucet().build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_rejection_containing_error(
        "The faucet doesn't have funds on this environment. Consider locking fee from an account instead."
    );
}

#[test]
fn fee_xrd_on_empty_faucet_should_give_nice_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| {
                    BabylonSettings::test_default().with_faucet_supply(Decimal::ZERO)
                })
                .from_bootstrap_to_latest()
        })
        .build();

    // Act
    let manifest = ManifestBuilder::new().get_free_xrd_from_faucet().build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_rejection_containing_error(
        "The faucet doesn't have funds on this environment. You will need to source XRD another way."
    );
}
