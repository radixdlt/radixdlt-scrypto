use radix_engine::kernel::call_frame::{ProcessSubstateError, WriteSubstateError};
use radix_engine::{
    errors::{CallFrameError, KernelError, RuntimeError},
    kernel::call_frame::MoveModuleError,
    types::*,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_create_global_node_with_local_ref() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reference");

    // Call function
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(
                package_address,
                "ReferenceTest",
                "create_global_node_with_local_ref",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::MoveModuleError(x),
        )) => {
            matches!(x, MoveModuleError::NonGlobalRefNotAllowed(_))
        }
        _ => false,
    });
}

#[test]
fn test_add_local_ref_to_stored_substate() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reference");

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(
                component_address,
                "add_local_ref_to_stored_substate",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::WriteSubstateError(x),
        )) => {
            matches!(
                x,
                WriteSubstateError::ProcessSubstateError(ProcessSubstateError::NonGlobalRefNotAllowed(..))
            )
        }
        _ => false,
    });
}
