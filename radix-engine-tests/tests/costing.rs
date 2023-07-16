use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_mint_1_nft_10_times_versus_10_nfts_once() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account1) = test_runner.new_allocated_account();
    let (_, _, account2) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/costing");
    let component_address = test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_function(package_address, "CostingTest", "init", manifest_args!())
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    // Act
    const N: u8 = 10;
    let receipt1 = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .then(|mut builder| {
                for _ in 0..N {
                    builder =
                        builder.call_method(component_address, "mint_1_nft", manifest_args!());
                }
                builder
            })
            .try_deposit_batch_or_abort(account1)
            .build(),
        vec![],
    );
    let receipt2 = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .call_method(component_address, "mint_n_nfts", manifest_args!(N))
            .try_deposit_batch_or_abort(account2)
            .build(),
        vec![],
    );

    // Assert
    let total_cost_1 = receipt1.expect_commit_success().fee_summary.total_cost();
    let total_cost_2 = receipt2.expect_commit_success().fee_summary.total_cost();
    assert_eq!(
        total_cost_1.round(2, RoundingMode::ToNearestMidpointTowardZero),
        dec!("0.60")
    );
    assert_eq!(
        total_cost_2.round(2, RoundingMode::ToNearestMidpointTowardZero),
        dec!("0.24")
    );
}
