use radix_engine::errors::{RuntimeError, SystemUpstreamError, VmError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn small_sbor_depth_should_succeed() {
    test_sbor_depth(10, true);
}

#[test]
#[cfg(not(feature = "wasmer"))]
fn large_sbor_depth_should_fail() {
    // Very Large Depth - we get a panic at encoding time in the Scrypto WASM
    test_sbor_depth(100, false)
}

fn test_sbor_depth(depth: usize, should_succeed: bool) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/deep_sbor");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "DeepAuthRulesOnCreate",
            "new",
            manifest_args!(XRD, depth),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    if should_succeed {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| matches!(e, RuntimeError::VmError(VmError::Wasm(..))));
        // TODO: (from Josh) This failure is currently flaking after rust update to 1.71. We need to further
        // investigate why this is occurring
        //receipt.expect_specific_failure(|f| f.to_string().contains("MaxDepthExceeded"));

        // TODO: (from David) I'd hoped for a third style of error - where scrypto can encode it but
        // It's an error when it's put in the substate
        // The change point is at a depth of 40/41, but I can't find this third kind of behaviour - likely because
        // scrypto actually encodes the full substate itself
    }
}

#[test]
fn setting_struct_with_small_sbor_depth_should_succeed() {
    test_setting_struct_with_deep_recursive_data_inside_component(10, true);
}

#[test]
#[cfg(not(feature = "wasmer"))]
fn setting_struct_with_very_large_sbor_depth_should_fail() {
    test_setting_struct_with_deep_recursive_data_inside_component(100, false);
}

fn test_setting_struct_with_deep_recursive_data_inside_component(
    depth: usize,
    should_succeed: bool,
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/deep_sbor");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "DeepStruct", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "set_depth", manifest_args!(XRD, depth))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    if should_succeed {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| matches!(e, RuntimeError::VmError(VmError::Wasm(..))));
        // TODO: (from Josh) This failure is currently flaking after rust update to 1.71. We need to further
        // investigate why this is occurring
        // receipt.expect_specific_failure(|f| f.to_string().contains("MaxDepthExceeded"));

        // TODO: (from David) I'd hoped for a third style of error - where scrypto can encode it but
        // It's an error when it's put in the substate
        // The change point is at a depth of 40/41, but I can't find this third kind of behaviour - likely because
        // scrypto actually encodes the full substate itself
    }
}

#[test]
fn malicious_component_replying_with_large_payload_is_handled_well_by_engine() {
    // Act 1 - Small Depth
    let receipt = publish_wasm_with_deep_sbor_response_and_execute_it(10);
    receipt.expect_commit_success();

    // Act 2 - Depth just under the limit
    let receipt = publish_wasm_with_deep_sbor_response_and_execute_it(SCRYPTO_SBOR_V1_MAX_DEPTH);
    receipt.expect_commit_success();

    // Act 2 - Depth just over the limit
    let receipt =
        publish_wasm_with_deep_sbor_response_and_execute_it(SCRYPTO_SBOR_V1_MAX_DEPTH + 1);
    receipt.expect_specific_failure(|f| {
        matches!(
            f,
            RuntimeError::SystemUpstreamError(SystemUpstreamError::OutputDecodeError(
                DecodeError::MaxDepthExceeded(_)
            ))
        )
    });
}

fn publish_wasm_with_deep_sbor_response_and_execute_it(depth: usize) -> TransactionReceipt {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let code = wat2wasm(
        &include_str!("wasm/deep_sbor_response.wat").replace("${depth}", &depth.to_string()),
    );
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    test_runner.execute_manifest(manifest, vec![])
}
