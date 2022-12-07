use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::{AuthModule, SystemInstruction, SystemTransaction};

#[test]
fn get_epoch_should_succeed() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "EpochManagerTest", "get_epoch", args![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_commit_success();
    let epoch: u64 = scrypto_decode(&outputs[1]).unwrap();
    assert_eq!(epoch, 0);
}

#[test]
fn set_epoch_without_supervisor_auth_fails() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let epoch = 9876u64;
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "EpochManagerTest",
            "set_epoch",
            args!(EPOCH_MANAGER, epoch),
        )
        .call_function(package_address, "EpochManagerTest", "get_epoch", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn epoch_manager_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let instructions = vec![SystemInstruction::CallNativeFunction {
        function_ident: NativeFunctionIdent {
            blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_owned(),
            function_name: EpochManagerFunction::Create.as_ref().to_owned(),
        },
        args: args!(),
    }
    .into()];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        &SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
        }
        .get_executable(vec![]),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn epoch_manager_create_should_succeed_with_system_privilege() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let instructions = vec![SystemInstruction::CallNativeFunction {
        function_ident: NativeFunctionIdent {
            blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_owned(),
            function_name: EpochManagerFunction::Create.as_ref().to_owned(),
        },
        args: args!(),
    }
    .into()];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        &SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
        }
        .get_executable(vec![AuthModule::system_role_non_fungible_address()]),
    );

    // Assert
    receipt.expect_commit_success();
}
