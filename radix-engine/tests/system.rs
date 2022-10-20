use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::AuthModule;

#[test]
fn get_epoch_should_succeed() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/system");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "SystemTest", "get_epoch", args![])
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
    let package_address = test_runner.compile_and_publish("./tests/system");

    // Act
    let epoch = 9876u64;
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "SystemTest",
            "set_epoch",
            args!(SYS_SYSTEM_COMPONENT, epoch),
        )
        .call_function(package_address, "SystemTest", "get_epoch", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn system_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10u32.into(), SYS_FAUCET_COMPONENT)
        .call_native_function("System", SystemFunction::Create.as_ref(), args!())
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![AuthModule::validator_role_nf_address()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn system_create_should_succeed_with_system_privilege() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_native_function("System", SystemFunction::Create.as_ref(), args!())
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![AuthModule::system_role_nf_address()]);

    // Assert
    receipt.expect_commit_success();
}
