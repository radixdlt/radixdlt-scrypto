use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::resource::{require, FromPublicKey};
use radix_engine_interface::rule;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_make_cross_component_process_call_dataout_authorization() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let auth = test_runner.create_non_fungible_resource(account);
    let auth_local_id = NonFungibleLocalId::integer(1);
    let auth_global_id = NonFungibleGlobalId::new(auth, auth_local_id);
    let authorization = AccessRulesConfig::new().method(
        "get_component_state",
        rule!(require(auth_global_id)),
        rule!(deny_all),
    );

    let package_address = test_runner.compile_and_publish("./tests/blueprints/component");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            manifest_args!(authorization),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let secured_component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let my_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn can_make_cross_component_process_call_data_authorization() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_local_id = NonFungibleLocalId::integer(1);
    let auth_global_id = NonFungibleGlobalId::new(auth, auth_local_id.clone());
    let authorization = AccessRulesConfig::new().method(
        "get_component_state",
        rule!(require(auth_global_id)),
        rule!(deny_all),
    );

    let package_address = test_runner.compile_and_publish("./tests/blueprints/component");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            manifest_args!(authorization),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let secured_component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let my_component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_non_fungibles_from_account(account, auth, &BTreeSet::from([auth_local_id]))
        .call_method(
            my_component,
            "put_auth",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn root_auth_zone_does_not_carry_over_cross_component_calls() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_local_id = NonFungibleLocalId::integer(1);
    let auth_global_id = NonFungibleGlobalId::new(auth, auth_local_id);
    let authorization = AccessRulesConfig::new().method(
        "get_component_state",
        rule!(require(auth_global_id)),
        rule!(deny_all),
    );

    let package_address = test_runner.compile_and_publish("./tests/blueprints/component");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            manifest_args!(authorization),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let secured_component = receipt.expect_commit(true).new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let my_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account, auth)
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}
