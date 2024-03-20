use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn mixed_up_modules_causes_type_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("module"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ComponentModule",
            "globalize_with_mixed_up_modules",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidModuleType { .. })
        )
    });
}
