use radix_engine::types::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_kv_store_with_many_large_keys() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_kernel_trace().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("storage"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "LargeKey",
            "create_kv_store_with_many_large_keys",
            manifest_args!(400u32),
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);
    assert_eq!(
        receipt.fee_summary.total_storage_cost_in_xrd,
        dec!("40.12508323306")
    );
}
