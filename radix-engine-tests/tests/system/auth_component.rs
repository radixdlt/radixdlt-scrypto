use radix_common::prelude::*;
use radix_engine_interface::blueprints::resource::require;
use radix_engine_interface::rule;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn create_secured_component(
    ledger: &mut DefaultLedgerSimulator,
    auth: NonFungibleGlobalId,
    package_address: PackageAddress,
) -> ComponentAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            manifest_args!(rule!(require(auth))),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let secured_component = receipt.expect_commit(true).new_component_addresses()[0];
    secured_component
}

fn create_resource_secured_component(
    ledger: &mut DefaultLedgerSimulator,
    account: ComponentAddress,
    package_address: PackageAddress,
) -> (ComponentAddress, NonFungibleGlobalId) {
    let auth = ledger.create_non_fungible_resource(account);
    let auth_local_id = NonFungibleLocalId::integer(1);
    let auth_global_id = NonFungibleGlobalId::new(auth, auth_local_id);
    let secured_component =
        create_secured_component(ledger, auth_global_id.clone(), package_address);
    (secured_component, auth_global_id)
}

fn create_component(
    ledger: &mut DefaultLedgerSimulator,
    package_address: PackageAddress,
) -> ComponentAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let my_component = receipt.expect_commit(true).new_component_addresses()[0];
    my_component
}

#[test]
fn cannot_make_cross_component_call_without_correct_global_caller_authorization() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));
    let badge =
        NonFungibleGlobalId::global_caller_badge(GlobalCaller::GlobalObject(account.into()));
    let secured_component = create_secured_component(&mut ledger, badge, package_address);
    let my_component = create_component(&mut ledger, package_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn can_make_cross_component_call_with_correct_global_caller_authorization() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));
    let my_component = create_component(&mut ledger, package_address);
    let badge =
        NonFungibleGlobalId::global_caller_badge(GlobalCaller::GlobalObject(my_component.into()));
    let secured_component = create_secured_component(&mut ledger, badge, package_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_make_cross_component_call_without_resource_authorization() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));
    let (secured_component, _) =
        create_resource_secured_component(&mut ledger, account, package_address);
    let my_component = create_component(&mut ledger, package_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn can_make_cross_component_call_with_resource_authorization() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));
    let (secured_component, auth_id) =
        create_resource_secured_component(&mut ledger, account, package_address);
    let my_component = create_component(&mut ledger, package_address);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(
            account,
            auth_id.resource_address(),
            [auth_id.local_id().clone()],
        )
        .call_method(
            my_component,
            "put_auth",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn root_auth_zone_does_not_carry_over_cross_component_calls() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));
    let (secured_component, auth_id) =
        create_resource_secured_component(&mut ledger, account, package_address);
    let my_component = create_component(&mut ledger, package_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(account, auth_id)
        .call_method(
            my_component,
            "cross_component_call",
            manifest_args!(secured_component),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}
