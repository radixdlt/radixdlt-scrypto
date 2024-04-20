#![allow(clippy::type_complexity)]

use radix_engine_tests::common::*;
use radix_native_sdk::resource::*;
use radix_substate_store_queries::typed_substate_layout::two_resource_pool::*;
use scrypto_test::prelude::*;

#[test]
fn kernel_modules_are_reset_after_calling_a_with_method() {
    // Arrange
    let mut env = TestEnvironment::new();
    let with_methods: &[fn(
        &mut TestEnvironment<InMemorySubstateDatabase>,
        fn(&mut TestEnvironment<InMemorySubstateDatabase>),
    )] = &[
        TestEnvironment::with_kernel_trace_module_enabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_limits_module_enabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_costing_module_enabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_auth_module_enabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_transaction_runtime_module_enabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_execution_trace_module_enabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_kernel_trace_module_disabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_limits_module_disabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_costing_module_disabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_auth_module_disabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_transaction_runtime_module_disabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
        TestEnvironment::with_execution_trace_module_disabled::<
            fn(&mut TestEnvironment<InMemorySubstateDatabase>),
            (),
        >,
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
fn auth_module_can_be_disabled_at_runtime() {
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
fn state_of_components_can_be_read() {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let rtn = env.with_component_state::<(Vault, Own), _, _, _>(FAUCET, |_, _| {});

    // Assert
    assert!(rtn.is_ok())
}

#[test]
fn can_invoke_owned_nodes_read_from_state() {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let amount = env
        .with_component_state::<(Vault, Own), _, _, _>(FAUCET, |(vault, _), env| vault.amount(env));

    // Assert
    amount
        .expect("Failed to get the vault amount")
        .expect("Failed to get the vault amount");
}

#[test]
fn references_read_from_state_are_visible_in_tests() {
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

    let code = include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm");
    let definition = manifest_decode(include_workspace_asset_bytes!(
        "radix-transaction-scenarios",
        "radiswap.rpd"
    ))
    .unwrap();

    let (radiswap_package, _) =
        PackageFactory::publish(code.to_vec(), definition, Default::default(), &mut env).unwrap();

    let radiswap_component = env
        .call_function_typed::<_, ComponentAddress>(
            radiswap_package,
            "Radiswap",
            "new",
            &(OwnerRole::None, resource1.0, resource2.0),
        )
        .unwrap();

    // Act
    let radiswap_pool_component = env
        .with_component_state::<(ComponentAddress,), _, _, _>(radiswap_component, |address, _| {
            address.0
        })
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

#[test]
fn references_read_from_state_are_visible_in_tests1() {
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

    let code = include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm");
    let definition = manifest_decode(include_workspace_asset_bytes!(
        "radix-transaction-scenarios",
        "radiswap.rpd"
    ))
    .unwrap();

    let (radiswap_package, _) =
        PackageFactory::publish(code.to_vec(), definition, Default::default(), &mut env).unwrap();

    let radiswap_component = env
        .call_function_typed::<_, ComponentAddress>(
            radiswap_package,
            "Radiswap",
            "new",
            &(OwnerRole::None, resource1.0, resource2.0),
        )
        .unwrap();

    let radiswap_pool_component = env
        .with_component_state::<(ComponentAddress,), _, _, _>(radiswap_component, |address, _| {
            address.0
        })
        .unwrap();

    // Act
    let amount = env
        .with_component_state::<VersionedTwoResourcePoolState, _, _, _>(
            radiswap_pool_component,
            |state, env| {
                let [(_, vault1), (_, _)] = &mut state.as_unique_version_mut().vaults;
                vault1.amount(env)
            },
        )
        .unwrap();

    // Assert
    amount.expect("Failed to get the vault amount");
}

#[test]
fn can_read_kv_entries_from_a_store_read_from_state() {
    // Arrange
    let mut env = TestEnvironment::new();
    let _ = env
        .call_method_typed::<_, _, Bucket>(FAUCET, "free", &())
        .unwrap();

    env.with_component_state::<(Vault, Own), _, _, _>(FAUCET, |(_, kv_store), env| {
        // Act
        let handle = env
            .key_value_store_open_entry(
                kv_store.as_node_id(),
                &scrypto_encode(&Hash([0; 32])).unwrap(),
                LockFlags::empty(),
            )
            .unwrap();
        let epoch = env.key_value_entry_get_typed::<Epoch>(handle).unwrap();
        env.key_value_entry_close(handle).unwrap();

        // Assert
        assert!(epoch.is_some())
    })
    .unwrap();
}

#[test]
fn can_get_and_set_epoch() {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    env.set_current_epoch(Epoch::of(200));

    // Assert
    assert_eq!(env.get_current_epoch().number(), 200)
}

#[test]
fn can_get_and_set_timestamp() {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    env.set_current_time(Instant::new(1692951060));

    // Assert
    assert_eq!(env.get_current_time(), Instant::new(1692951060))
}

#[test]
fn creation_of_mock_fungible_buckets_succeeds() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let bucket = BucketFactory::create_fungible_bucket(XRD, 10.into(), Mock, &mut env)?;

    // Assert
    let amount = bucket.amount(&mut env)?;
    assert_eq!(amount, dec!("10"));

    Ok(())
}

#[test]
fn creation_of_mock_non_fungible_buckets_succeeds() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let bucket = BucketFactory::create_non_fungible_bucket(
        ACCOUNT_OWNER_BADGE,
        btreemap!(NonFungibleLocalId::bytes(vec![0x00]).unwrap() => ("Hello", GENESIS_HELPER)),
        Mock,
        &mut env,
    )?;

    // Assert
    let amount = bucket.amount(&mut env)?;
    assert_eq!(amount, dec!("1"));

    Ok(())
}

#[test]
fn creation_of_disable_auth_and_mint_fungible_buckets_succeeds() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let bucket =
        BucketFactory::create_fungible_bucket(XRD, 10.into(), DisableAuthAndMint, &mut env)?;

    // Assert
    let amount = bucket.amount(&mut env)?;
    assert_eq!(amount, dec!("10"));

    Ok(())
}
