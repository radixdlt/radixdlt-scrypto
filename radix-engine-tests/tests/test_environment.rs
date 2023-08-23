use native_sdk::resource::*;
use scrypto_test::prelude::*;

#[test]
pub fn kernel_modules_are_reset_after_calling_a_with_method() {
    // Arrange
    let mut env = TestEnvironment::new();
    let with_methods: &[fn(&mut TestEnvironment, fn(&mut TestEnvironment))] = &[
        TestEnvironment::with_kernel_trace_module_enabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_limits_module_enabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_costing_module_enabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_auth_module_enabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_transaction_runtime_module_enabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_execution_trace_module_enabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_kernel_trace_module_disabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_limits_module_disabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_costing_module_disabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_auth_module_disabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_transaction_runtime_module_disabled::<fn(&mut TestEnvironment), ()>,
        TestEnvironment::with_execution_trace_module_disabled::<fn(&mut TestEnvironment), ()>,
    ];

    for method in with_methods {
        let enabled_modules = env.enabled_modules();

        // Act
        method(&mut env, |_| {});

        // Assert
        assert_eq!(enabled_modules, env.enabled_modules())
    }
}

#[test]
pub fn auth_module_can_be_disabled_at_runtime() {
    // Arrange
    let mut env = TestEnvironment::new();
    env.with_auth_module_disabled(|env| {
        // Act
        let rtn = ResourceManager(XRD).mint_fungible(1.into(), env);

        // Assert
        assert!(rtn.is_ok())
    })
}

#[test]
pub fn state_of_components_can_be_read() {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let rtn = env.read_component_state::<(Vault, Own)>(FAUCET);

    // Assert
    assert!(rtn.is_ok())
}

#[test]
#[ignore = "Currently fails because the test call-frame has no references to the owned nodes we read."]
pub fn can_invoke_owned_nodes_read_from_state() {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let (vault, _) = env
        .read_component_state::<(Vault, Own)>(FAUCET)
        .expect("Should succeed");

    // Assert
    assert!(vault.amount(&mut env).is_ok())
}

#[test]
pub fn references_read_from_state_are_visible_in_tests() {
    // Arrange
    let mut env = TestEnvironment::new();

    let resource1 = ResourceManager::new_fungible(
        OwnerRole::None,
        false,
        18,
        Default::default(),
        MetadataInit::default(),
        None,
        &mut env,
    )
    .unwrap();
    let resource2 = ResourceManager::new_fungible(
        OwnerRole::None,
        false,
        18,
        Default::default(),
        MetadataInit::default(),
        None,
        &mut env,
    )
    .unwrap();

    let radiswap_package =
        Package::compile_and_publish("../assets/blueprints/radiswap", &mut env).unwrap();

    let radiswap_component = env
        .call_function_typed::<_, ComponentAddress>(
            radiswap_package,
            "Radiswap",
            "new",
            &(OwnerRole::None, resource1.0, resource2.0),
        )
        .unwrap();

    // Act
    let (radiswap_pool_component,) = env
        .read_component_state::<(ComponentAddress,)>(radiswap_component)
        .unwrap();

    // Assert
    assert!(env
        .call_method_typed::<_, _, TwoResourcePoolGetVaultAmountsOutput>(
            radiswap_pool_component,
            TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
            &TwoResourcePoolGetVaultAmountsInput {}
        )
        .is_ok())
}
