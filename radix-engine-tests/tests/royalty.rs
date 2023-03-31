use radix_engine::{system::kernel_modules::costing::u128_to_decimal, types::*};
use radix_engine_constants::DEFAULT_COST_UNIT_PRICE;
use radix_engine_interface::blueprints::resource::FromPublicKey;
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
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address: ComponentAddress = receipt.expect_commit(true).output(1);

    // Call the paid method
    let account_pre_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let commit_result = receipt.expect_commit(true);
    assert_eq!(
        commit_result.fee_summary.total_royalty_cost_xrd,
        dec!("1") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE)
    );
    let account_post_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let component_royalty = test_runner
        .inspect_component_royalty(component_address)
        .unwrap();
    assert_eq!(
        account_pre_balance - account_post_balance,
        commit_result.fee_summary.total_execution_cost_xrd
            + commit_result.fee_summary.total_royalty_cost_xrd
    );
    assert_eq!(
        component_royalty,
        commit_result.fee_summary.total_royalty_cost_xrd
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
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty_enabled",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address: ComponentAddress = receipt.expect_commit(true).output(1);

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
        package_address,
        component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    let account_pre_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let commit_result = receipt.expect_commit(true);
    assert_eq!(
        commit_result.fee_summary.total_royalty_cost_xrd,
        (dec!("1") + dec!("2")) * u128_to_decimal(DEFAULT_COST_UNIT_PRICE)
    );
    let account_post_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let package_royalty = test_runner
        .inspect_package_royalty(package_address)
        .unwrap();
    let component_royalty = test_runner
        .inspect_component_royalty(component_address)
        .unwrap();
    assert_eq!(
        account_pre_balance - account_post_balance,
        commit_result.fee_summary.total_execution_cost_xrd
            + commit_result.fee_summary.total_royalty_cost_xrd
    );
    assert_eq!(
        package_royalty,
        dec!("2") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE)
    );
    assert_eq!(
        component_royalty,
        dec!("1") * u128_to_decimal(DEFAULT_COST_UNIT_PRICE)
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
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit(true);
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
            .call_method(component_address, "paid_method_panic", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_failure();
    assert_eq!(test_runner.inspect_package_royalty(package_address), None);
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        None
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
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);
    receipt.expect_commit(true);
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
                manifest_args!(package_address),
            )
            .call_method(
                account,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

    // Claim component royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "claim_component_royalty",
                manifest_args!(component_address),
            )
            .call_method(
                account,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

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
