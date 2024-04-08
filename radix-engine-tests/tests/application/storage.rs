use radix_common::*;
use radix_common::data::manifest::*;
use radix_common::prelude::*;
use radix_engine_interface::*;
use radix_engine_interface::api::*;
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use radix_transactions::builder::*;
use scrypto_test::ledger_simulator::*;

#[test]
fn test_kv_store_with_many_large_keys() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("storage"));

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

    let receipt = ledger.execute_manifest(manifest, vec![]);
    assert_eq!(
        receipt.fee_summary.total_storage_cost_in_xrd,
        dec!("40.12508323306")
    );
}
