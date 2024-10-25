use radix_engine_tests::common::PackageLoader;
use scrypto_test::prelude::*;

#[test]
fn test_create_buffers_within_limits() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("wasm_limits"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "WasmLimits", "create_buffers", (4usize,))
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_crate_buffers_beyond_limits() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("wasm_limits"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "WasmLimits", "create_buffers", (5usize,))
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::TooManyBuffers))
        )
    })
}
