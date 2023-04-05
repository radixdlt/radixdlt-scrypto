use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

/// This tests the external_blueprint! and external_component! macros
#[test]
fn test_external_bridges() {
    // ARRANGE
    let mut test_runner = TestRunner::builder().build();

    // Part 1 - Upload the target and caller packages
    // Note - we put them in separate packages so that we test that the package call is to an external package
    let target_package_address = test_runner.compile_and_publish("./tests/blueprints/component");
    let caller_package_address =
        test_runner.compile_and_publish("./tests/blueprints/external_blueprint_caller");

    // Part 2 - Get a target component address
    let manifest1 = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            target_package_address,
            "ExternalBlueprintTarget",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();

    let target_component_address = receipt1.expect_commit(true).new_component_addresses()[0];

    // Part 3 - Get the caller component address
    let manifest2 = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            caller_package_address,
            "ExternalBlueprintCaller",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt2 = test_runner.execute_manifest(manifest2, vec![]);
    receipt2.expect_commit_success();

    let caller_component_address = receipt2.expect_commit(true).new_component_addresses()[0];

    // ACT
    let manifest3 = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_method(
            caller_component_address,
            "run_tests_with_external_blueprint",
            manifest_args!(target_package_address),
        )
        .build();
    let receipt3 = test_runner.execute_manifest(manifest3, vec![]);

    // ASSERT
    receipt3.expect_commit_success();

    // ACT
    let manifest4 = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_method(
            caller_component_address,
            "run_tests_with_external_component",
            manifest_args!(target_component_address),
        )
        .build();
    let receipt4 = test_runner.execute_manifest(manifest4, vec![]);

    // ASSERT
    receipt4.expect_commit_success();
}
