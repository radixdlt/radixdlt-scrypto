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
    let (code, abi) = test_runner.compile("./tests/blueprints/royalty-auth");
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .publish_package_with_owner(code, abi)
            .call_method(
                account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    let package_address = receipt.expect_commit().entity_changes.new_package_addresses[0];

    // Enable package royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .create_proof_from_account(account, ENTITY_OWNER_TOKEN)
            .call_native_method(
                RENodeId::Global(GlobalAddress::Package(package_address)),
                "set_royalty_config",
                args!(
                    package_address,
                    HashMap::from([(
                        "RoyaltyTest".to_owned(),
                        RoyaltyConfigBuilder::new()
                            .add_rule("paid_method", dec!("0.2"))
                            .add_rule("paid_method_panic", dec!("0.2"))
                            .default(dec!("0")),
                    )])
                ),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .create_proof_from_account(account, ENTITY_OWNER_TOKEN)
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty_enabled",
                args!(),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

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
    let (mut store, account, public_key, package_address, _component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, ENTITY_OWNER_TOKEN)
            .call_native_method(
                RENodeId::Global(GlobalAddress::Package(package_address)),
                "set_royalty_config",
                args!(
                    package_address,
                    HashMap::from([(
                        "RoyaltyTest".to_owned(),
                        RoyaltyConfigBuilder::new().default(dec!("0")),
                    )])
                ),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Negative case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_native_method(
                RENodeId::Global(GlobalAddress::Package(package_address)),
                "set_royalty_config",
                args!(
                    package_address,
                    HashMap::from([(
                        "RoyaltyTest".to_owned(),
                        RoyaltyConfigBuilder::new().default(dec!("0")),
                    )])
                ),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_only_package_owner_can_claim_royalty() {
    let (mut store, account, public_key, package_address, _component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, ENTITY_OWNER_TOKEN)
            .call_native_method(
                RENodeId::Global(GlobalAddress::Package(package_address)),
                "claim_royalty",
                args!(package_address),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Negative case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_native_method(
                RENodeId::Global(GlobalAddress::Package(package_address)),
                "claim_royalty",
                args!(package_address),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_only_component_owner_can_set_royalty_config() {
    let (mut store, account, public_key, _package_address, component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, ENTITY_OWNER_TOKEN)
            .call_native_method(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                "set_royalty_config",
                args!(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    RoyaltyConfigBuilder::new().default(dec!("0"))
                ),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Negative case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_native_method(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                "set_royalty_config",
                args!(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    RoyaltyConfigBuilder::new().default(dec!("0"))
                ),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_only_component_owner_can_claim_royalty() {
    let (mut store, account, public_key, _package_address, component_address) =
        set_up_package_and_component();
    let mut test_runner = TestRunner::new(true, &mut store);

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, ENTITY_OWNER_TOKEN)
            .call_native_method(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                "claim_royalty",
                args!(RENodeId::Global(GlobalAddress::Component(
                    component_address
                ))),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Negative case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .call_native_method(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                "claim_royalty",
                args!(RENodeId::Global(GlobalAddress::Component(
                    component_address
                ))),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}
