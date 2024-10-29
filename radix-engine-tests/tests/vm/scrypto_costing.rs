use radix_common::prelude::*;
use radix_engine::updates::ProtocolVersion;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn can_call_usd_price() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("costing"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CostingTest",
            "usd_price",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn usd_price_costing_after_protocol_update() {
    // Call usd_price() function before protocol update
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_cost_breakdown()
        .with_custom_protocol(|builder| builder.only_babylon())
        .build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("costing"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CostingTest",
            "usd_price",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    println!("DONE");

    // Store execution cost
    let cost_before_update = receipt.fee_summary.total_execution_cost_units_consumed;

    // Call usd_price() function after Bottlenose protocol update
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_cost_breakdown()
        .with_custom_protocol(|builder: radix_engine::updates::ProtocolBuilder| {
            builder.from_bootstrap_to(ProtocolVersion::Bottlenose)
        })
        .build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("costing"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CostingTest",
            "usd_price",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    assert!(receipt
        .fee_details
        .unwrap()
        .execution_cost_breakdown
        .get(&ExecutionCostingEntry::QueryFeeReserve.to_trace_key())
        .is_some());
    assert_eq!(
        cost_before_update + // usd_price() call cost includes also
            80024 +          // RefCheck
            500, // QueryFeeReserve
        receipt.fee_summary.total_execution_cost_units_consumed
    );
}
