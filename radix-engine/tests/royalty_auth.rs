use radix_engine::types::*;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::node::NetworkDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn set_up_package_and_component() -> (
    TestRunner,
    ComponentAddress,
    EcdsaSecp256k1PublicKey,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
) {
    // Basic setup
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_allocated_account();
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr = NonFungibleAddress::new(owner_badge_resource, NonFungibleId::U32(1));

    // Publish package
    let (code, abi) = Compile::compile("./tests/blueprints/royalty-auth");
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .publish_package_with_owner(code, abi, owner_badge_addr.clone())
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    let package_address = receipt.expect_commit().entity_changes.new_package_addresses[0];

    // Enable package royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .create_proof_from_account(account, owner_badge_resource)
            .set_package_royalty_config(
                package_address,
                BTreeMap::from([(
                    "RoyaltyTest".to_owned(),
                    RoyaltyConfigBuilder::new()
                        .add_rule("paid_method", 2)
                        .add_rule("paid_method_panic", 2)
                        .default(0),
                )]),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .create_proof_from_account(account, owner_badge_resource)
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty_enabled",
                args!(owner_badge_addr),
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
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    (
        test_runner,
        account,
        public_key,
        package_address,
        component_address,
        owner_badge_resource,
    )
}

#[test]
fn test_only_package_owner_can_set_royalty_config() {
    let (
        mut test_runner,
        account,
        public_key,
        package_address,
        _component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, owner_badge_resource)
            .set_package_royalty_config(
                package_address,
                BTreeMap::from([(
                    "RoyaltyTest".to_owned(),
                    RoyaltyConfigBuilder::new().default(0),
                )]),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Negative case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .set_package_royalty_config(
                package_address,
                BTreeMap::from([(
                    "RoyaltyTest".to_owned(),
                    RoyaltyConfigBuilder::new().default(0),
                )]),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_only_package_owner_can_claim_royalty() {
    let (
        mut test_runner,
        account,
        public_key,
        package_address,
        _component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, owner_badge_resource)
            .claim_package_royalty(package_address)
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
            .claim_package_royalty(package_address)
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
    let (
        mut test_runner,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, owner_badge_resource)
            .set_component_royalty_config(component_address, RoyaltyConfigBuilder::new().default(0))
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Negative case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .set_component_royalty_config(component_address, RoyaltyConfigBuilder::new().default(0))
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_only_component_owner_can_claim_royalty() {
    let (
        mut test_runner,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, owner_badge_resource)
            .claim_component_royalty(component_address)
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
            .claim_component_royalty(component_address)
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
