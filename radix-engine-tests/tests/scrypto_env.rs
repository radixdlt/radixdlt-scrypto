use radix_engine::errors::{CallFrameError, KernelError, RuntimeError, SystemError};
use radix_engine::kernel::call_frame::OpenSubstateError;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn should_not_be_able_to_node_create_with_invalid_blueprint() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/scrypto_env");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ScryptoEnvTest",
            "create_node_with_invalid_blueprint",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::BlueprintDoesNotExist(..)) => true,
        _ => false,
    });
}

#[test]
fn should_not_be_able_to_open_mut_substate_twice_if_object_in_heap() {
    should_not_be_able_to_open_mut_substate_twice(true);
}

#[test]
fn should_not_be_able_to_open_mut_substate_twice_if_object_globalized() {
    should_not_be_able_to_open_mut_substate_twice(false);
}

fn should_not_be_able_to_open_mut_substate_twice(heap: bool) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/scrypto_env");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ScryptoEnvTest",
            "create_and_open_mut_substate_twice",
            manifest_args!(heap),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::OpenSubstateError(OpenSubstateError::SubstateLocked(..)),
        )) => true,
        _ => false,
    });
}
