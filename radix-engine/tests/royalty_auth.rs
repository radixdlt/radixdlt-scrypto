use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn set_up_package_and_component() -> (
    TypedInMemorySubstateStore,
    ComponentAddress,
    EcdsaSecp256k1PublicKey,
    PackageAddress,
    ComponentAddress,
) {
    // Basic setup
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/royalty");

    // Enable package royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "enable_royalty_for_this_package",
                args!(),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty_enabled",
                args!(),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    let component_address: ComponentAddress =
        scrypto_decode(&receipt.expect_commit_success()[1]).unwrap();

    (
        store,
        account,
        public_key,
        package_address,
        component_address,
    )
}

#[test]
fn test_only_package_owner_can_set_royalty_config() {
    let (mut store, account, public_key, _package_address, component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    assert_eq!(
        receipt.execution.fee_summary.royalty,
        dec!("0.1") + dec!("0.2"),
    );
}

#[test]
fn test_only_package_owner_can_claim_royalty() {
    let (mut store, account, public_key, _package_address, component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    assert_eq!(
        receipt.execution.fee_summary.royalty,
        dec!("0.1") + dec!("0.2"),
    );
}

#[test]
fn test_only_component_owner_can_set_royalty_config() {
    let (mut store, account, public_key, _package_address, component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    assert_eq!(
        receipt.execution.fee_summary.royalty,
        dec!("0.1") + dec!("0.2"),
    );
}

#[test]
fn test_only_component_owner_can_claim_royalty() {
    let (mut store, account, public_key, _package_address, component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    assert_eq!(
        receipt.execution.fee_summary.royalty,
        dec!("0.1") + dec!("0.2"),
    );
}
