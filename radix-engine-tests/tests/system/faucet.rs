mod package_loader;

use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn lock_fee_on_empty_faucet_should_give_nice_erro() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_genesis(CustomGenesis::with_faucet_supply(Decimal::ZERO))
        .build();

    // Act
    let manifest = ManifestBuilder::new().lock_fee_from_faucet().build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let rejection = receipt.expect_rejection();
    assert!(rejection.to_string().contains("The faucet doesn't have funds on this environment. Consider locking fee from an account instead."));
}

#[test]
fn fee_xrd_on_empty_faucet_should_give_nice_error() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_genesis(CustomGenesis::with_faucet_supply(Decimal::ZERO))
        .build();

    // Act
    let manifest = ManifestBuilder::new().get_free_xrd_from_faucet().build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let rejection = receipt.expect_rejection();
    assert!(rejection.to_string().contains("The faucet doesn't have funds on this environment. You will need to source XRD another way."));
}
