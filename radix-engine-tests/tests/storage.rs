use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_kv_store_with_many_large_keys() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/storage");

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
        receipt
            .fee_summary
            .total_storage_cost_in_xrd
            .safe_div(receipt.costing_parameters.state_storage_price)
            .unwrap(),
        dec!("419292")
    );
}
