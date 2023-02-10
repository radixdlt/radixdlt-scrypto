use radix_engine::fee::u128_to_decimal;
use radix_engine::types::*;
use radix_engine_constants::DEFAULT_COST_UNIT_PRICE;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_component_royalty() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/royalty");

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty_enabled",
                args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address: ComponentAddress = receipt.output(1);

    // Call the paid method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![],
    );

    receipt.expect_commit_success();
    assert_eq!(
        receipt.execution.fee_summary.total_royalty_cost_xrd,
        dec!("1") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE)
    );
}

fn set_up_package_and_component() -> (
    TestRunner,
    ComponentAddress,
    EcdsaSecp256k1PublicKey,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
) {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address =
        test_runner.compile_and_publish_with_owner("./tests/blueprints/royalty", owner_badge_addr);

    // Enable package royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .create_proof_from_account(account, owner_badge_resource)
            .call_function(
                package_address,
                "RoyaltyTest",
                "enable_royalty_for_this_package",
                args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty_enabled",
                args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address: ComponentAddress = receipt.output(1);

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
fn test_package_royalty() {
    let (
        mut test_runner,
        account,
        public_key,
        _package_address,
        component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    assert_eq!(
        receipt.execution.fee_summary.total_royalty_cost_xrd,
        (dec!("1") + dec!("2")) * u128_to_decimal(DEFAULT_COST_UNIT_PRICE)
    );
}

#[test]
fn test_royalty_accumulation_when_success() {
    let (
        mut test_runner,
        account,
        public_key,
        package_address,
        component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_package_royalty(package_address),
        Some(dec!("2") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE))
    );
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        Some(dec!("1") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE))
    );
}

#[test]
fn test_royalty_accumulation_when_failure() {
    let (
        mut test_runner,
        account,
        public_key,
        package_address,
        component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method_panic", args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_failure();
    assert_eq!(
        test_runner.inspect_package_royalty(package_address),
        Some(dec!("0"))
    );
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        Some(dec!("0"))
    );
}

#[test]
fn test_claim_royalty() {
    let (
        mut test_runner,
        account,
        public_key,
        package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_package_royalty(package_address),
        Some(dec!("2") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE))
    );
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        Some(dec!("1") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE))
    );

    // Claim package royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .create_proof_from_account(account, owner_badge_resource)
            .call_function(
                package_address,
                "RoyaltyTest",
                "claim_package_royalty",
                args!(package_address),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Claim component royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "claim_component_royalty",
                args!(component_address),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // assert nothing left
    assert_eq!(
        test_runner.inspect_package_royalty(package_address),
        Some(dec!("0"))
    );
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        Some(dec!("0"))
    );
}
