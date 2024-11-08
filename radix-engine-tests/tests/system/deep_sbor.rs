use radix_common::prelude::*;
use radix_engine::transaction::TransactionReceipt;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
#[ignore = "TODO: investigate how the compiled wasm is producing unreachable"]
fn deep_auth_rules_on_component_create_creation_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("deep_sbor"));

    // Act 1 - Small Depth
    let depth = 10usize;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "DeepAuthRulesOnCreate",
            "new",
            manifest_args!(XRD, depth),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act 2 - Very Large Depth - we get a panic at encoding time in the Scrypto WASM
    let depth = 100usize;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "DeepAuthRulesOnCreate",
            "new",
            manifest_args!(XRD, depth),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_failure_containing_error("MaxDepthExceeded");

    // Act 3 - I'd hoped for a third style of error - where scrypto can encode it but
    //         It's an error when it's put in the substate
    //         The change point is at a depth of 40/41, but I can't find this third kind of behaviour - likely because
    //         scrypto actually encodes the full substate itself
}

#[test]
#[ignore = "TODO: investigate how the compiled wasm is producing unreachable"]
fn setting_struct_with_deep_recursive_data_panics_inside_component() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("deep_sbor"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "DeepStruct", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act 1 - Small Depth - Succeeds
    let depth = 10usize;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "set_depth", manifest_args!(XRD, depth))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act 2 - Very Large Depth - we get a panic at encoding time in the Scrypto WASM
    let depth = 100usize;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "set_depth", manifest_args!(XRD, depth))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_failure_containing_error("MaxDepthExceeded");

    // Act 3 - I'd hoped for a third style of error - where scrypto can encode it but
    //         It's an error when it's put in the substate
    //         The change point is at a depth of 42/43, but I can't find this third kind of behaviour.
}

#[test]
fn malicious_component_replying_with_large_payload_is_handled_well_by_engine() {
    // Act 1 - Small Depth
    let receipt = publish_wasm_with_deep_sbor_response_and_execute_it(10);
    receipt.expect_commit_success();

    // Act 2 - Depth just under the limit
    let receipt = publish_wasm_with_deep_sbor_response_and_execute_it(BLUEPRINT_PAYLOAD_MAX_DEPTH);
    receipt.expect_commit_success();

    // Act 2 - Depth just over the limit
    let receipt =
        publish_wasm_with_deep_sbor_response_and_execute_it(BLUEPRINT_PAYLOAD_MAX_DEPTH + 1);
    receipt.expect_specific_failure(|f| format!("{:?}", f).contains("MaxDepthExceeded"));
}

fn publish_wasm_with_deep_sbor_response_and_execute_it(depth: usize) -> TransactionReceipt {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let code = wat2wasm(
        &include_local_wasm_str!("deep_sbor_response.wat").replace("${depth}", &depth.to_string()),
    );
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    ledger.execute_manifest(manifest, vec![])
}
