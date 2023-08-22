use native_sdk::resource::*;
use scrypto_test::prelude::*;

#[test]
fn test_runtime_can_be_created() {
    TestRuntime::default();
}

#[test]
fn privileged_action_can_be_performed_when_auth_module_is_disabled() {
    // Arrange
    let mut test_runtime = TestRuntime::new();
    let mut resource_manager = ResourceManager(XRD);

    // Act
    let bucket = test_runtime
        .with_auth_module_disabled(|test_runtime| {
            resource_manager.mint_fungible(100.into(), test_runtime)
        })
        .unwrap();

    // Assert
    assert_eq!(bucket.amount(&mut test_runtime).unwrap(), dec!("100"));
}

#[test]
fn with_module_disabled_resets_modules_when_callback_is_finished() {
    // Arrange
    let mut test_runtime = TestRuntime::new();
    let mut resource_manager = ResourceManager(XRD);

    // Act
    test_runtime.with_auth_module_disabled(|_| {});
    let rtn = resource_manager.mint_fungible(100.into(), &mut test_runtime);

    // Assert
    assert!(matches!(
        rtn,
        Err(RuntimeError::SystemModuleError(
            SystemModuleError::AuthError(..)
        ))
    ))
}
