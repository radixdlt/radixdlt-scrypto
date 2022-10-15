use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_make_cross_component_call_without_authorization() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (_, _, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account);
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id);
    let authorization =
        AccessRules::new().method("get_component_state", rule!(require(auth_address.clone())));

    let package_address = test_runner.compile_and_publish("./tests/component");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            args!(authorization),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let secured_component = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let my_component = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            my_component,
            "cross_component_call",
            args!(secured_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn can_make_cross_component_call_with_authorization() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id.clone());
    let authorization =
        AccessRules::new().method("get_component_state", rule!(require(auth_address.clone())));

    let package_address = test_runner.compile_and_publish("./tests/component");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            args!(authorization),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let secured_component = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let my_component = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_ids(&BTreeSet::from([auth_id.clone()]), auth, account)
        .call_method(
            my_component,
            "put_auth",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            my_component,
            "cross_component_call",
            args!(secured_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
