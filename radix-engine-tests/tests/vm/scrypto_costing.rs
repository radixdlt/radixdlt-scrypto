use radix_engine_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn can_call_usd_price() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("costing"));

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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
