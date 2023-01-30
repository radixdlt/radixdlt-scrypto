use radix_engine::engine::{InterpreterError, KernelError, RuntimeError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::data::MAX_SCRYPTO_SBOR_DEPTH;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn deep_auth_rules_on_component_create_creation_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/deep_sbor");

    // Act 1 - Small Depth
    let depth = 10u8;
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "DeepAuthRulesOnCreate",
            "new",
            args!(RADIX_TOKEN, depth),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act 2 - Very Large Depth - we get a panic at encoding time in the Scrypto WASM
    let depth = 100u8;
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "DeepAuthRulesOnCreate",
            "new",
            args!(RADIX_TOKEN, depth),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|f| {
        matches!(
            f,
            RuntimeError::KernelError(KernelError::WasmRuntimeError(_))
        )
    });

    // Act 3 - I'd hoped for a third style of error - where scrypto can encode it but
    //         It's an error when it's put in the substate
    //         The change point is at a depth of 40/41, but I can't find this third kind of behaviour - likely because
    //         scrypto actually encodes the full substate itself
}

#[test]
fn setting_struct_with_deep_recursive_data_panics_inside_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/deep_sbor");

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "DeepStruct", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt.new_component_addresses().get(0).unwrap();

    // Act 1 - Small Depth - Succeeds
    let depth = 10u8;
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(*component_address, "set_depth", args!(RADIX_TOKEN, depth))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act 2 - Very Large Depth - we get a panic at encoding time in the Scrypto WASM
    let depth = 100u8;
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(*component_address, "set_depth", args!(RADIX_TOKEN, depth))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|f| {
        matches!(
            f,
            RuntimeError::KernelError(KernelError::WasmRuntimeError(_))
        )
    });

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
    let receipt = publish_wasm_with_deep_sbor_response_and_execute_it(MAX_SCRYPTO_SBOR_DEPTH);
    receipt.expect_commit_success();

    // Act 2 - Depth just over the limit
    let receipt = publish_wasm_with_deep_sbor_response_and_execute_it(MAX_SCRYPTO_SBOR_DEPTH + 1);
    receipt.expect_specific_failure(|f| {
        matches!(
            f,
            RuntimeError::InterpreterError(InterpreterError::InvalidScryptoReturn(
                DecodeError::MaxDepthExceeded(_)
            ))
        )
    });
}

fn publish_wasm_with_deep_sbor_response_and_execute_it(depth: u8) -> TransactionReceipt {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    let code = wat2wasm(
        &include_str!("wasm/deep_sbor_response.wat").replace("${depth}", &depth.to_string()),
    );
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi("Test", "f", Type::Any),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    test_runner.execute_manifest(manifest, vec![])
}
