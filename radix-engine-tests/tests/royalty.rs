use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::system::node_modules::royalty::ComponentRoyaltyError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_component_royalty() {
    // Arrange
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/royalty");

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
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

    // Act
    // Call the paid method
    let account_pre_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let commit_result = receipt.expect_commit(true);
    assert_eq!(commit_result.fee_summary.total_royalty_cost_xrd, dec!("3"));
    let account_post_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let component_royalty = test_runner.inspect_component_royalty(component_address);
    assert_eq!(
        account_pre_balance - account_post_balance,
        commit_result.fee_summary.total_cost()
    );
    assert_eq!(
        component_royalty,
        commit_result.fee_summary.total_royalty_cost_xrd - dec!("2"),
    );
}

#[test]
fn test_component_royalty_in_usd() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/royalty");

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
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
            .lock_fee(account, 500u32.into())
            .call_method(component_address, "paid_method_usd", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let commit_result = receipt.expect_commit(true);
    assert_eq!(
        commit_result.fee_summary.total_royalty_cost_xrd,
        dec!("1") * Decimal::try_from(DEFAULT_USD_PRICE_IN_XRD).unwrap()
    );
    let account_post_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let component_royalty = test_runner.inspect_component_royalty(component_address);
    assert_eq!(
        account_pre_balance - account_post_balance,
        commit_result.fee_summary.total_cost()
    );
    assert_eq!(
        component_royalty,
        commit_result.fee_summary.total_royalty_cost_xrd
    );
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
            .lock_fee(account, 500u32.into())
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let commit_result = receipt.expect_commit(true);
    assert_eq!(
        commit_result.fee_summary.total_royalty_cost_xrd,
        dec!("1") + dec!("2")
    );
    let account_post_balance = test_runner.account_balance(account, RADIX_TOKEN).unwrap();
    let package_royalty = test_runner
        .inspect_package_royalty(package_address)
        .unwrap();
    let component_royalty = test_runner.inspect_component_royalty(component_address);
    assert_eq!(
        account_pre_balance - account_post_balance,
        commit_result.fee_summary.total_cost()
    );
    assert_eq!(package_royalty, dec!("2"));
    assert_eq!(component_royalty, dec!("1"));
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
            .lock_fee(account, 500u32.into())
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit(true);
    assert_eq!(
        test_runner.inspect_package_royalty(package_address),
        Some(dec!("2"))
    );
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        dec!("1")
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
            .lock_fee(account, 500u32.into())
            .call_method(component_address, "paid_method_panic", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_failure();
    assert_eq!(
        test_runner.inspect_package_royalty(package_address),
        Some(Decimal::zero())
    );
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        Decimal::zero()
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
            .lock_fee(account, 500u32.into())
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);
    receipt.expect_commit(true);
    assert_eq!(
        test_runner.inspect_package_royalty(package_address),
        Some(dec!("2"))
    );
    assert_eq!(
        test_runner.inspect_component_royalty(component_address),
        dec!("1")
    );

    // Claim package royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                &btreeset!(NonFungibleLocalId::integer(1)),
            )
            .claim_package_royalties(package_address)
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

    // Claim component royalty
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .claim_component_royalties(component_address)
            .call_method(
                account,
                "try_deposit_batch_or_abort",
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
        dec!("0")
    );
}

fn cannot_initialize_package_royalty_if_greater_than_allowed(royalty_amount: RoyaltyAmount) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_public_key, _, account) = test_runner.new_allocated_account();
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));

    // Act
    let (code, mut definition) = Compile::compile("./tests/blueprints/royalty");
    let blueprint_def = definition.blueprints.get_mut("RoyaltyTest").unwrap();
    match &mut blueprint_def.royalty_config {
        PackageRoyaltyConfig::Enabled(royalties) => {
            for royalty in royalties.values_mut() {
                *royalty = royalty_amount.clone();
            }
        }
        PackageRoyaltyConfig::Disabled => {}
    }
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .publish_package_with_owner(code, definition, owner_badge_addr)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::RoyaltyAmountIsGreaterThanAllowed { .. }
            ))
        )
    });
}

#[test]
fn cannot_initialize_package_royalty_if_greater_xrd_than_allowed() {
    let max_royalty_allowed_in_xrd =
        Decimal::try_from(DEFAULT_MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
    let royalty_amount = RoyaltyAmount::Xrd(max_royalty_allowed_in_xrd + dec!("1"));
    cannot_initialize_package_royalty_if_greater_than_allowed(royalty_amount);
}

#[test]
fn cannot_initialize_package_royalty_if_greater_usd_than_allowed() {
    let max_royalty_allowed_in_xrd =
        Decimal::try_from(DEFAULT_MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
    let max_royalty_allowed_in_usd =
        max_royalty_allowed_in_xrd / Decimal::try_from(DEFAULT_USD_PRICE_IN_XRD).unwrap();
    let royalty_amount = RoyaltyAmount::Usd(max_royalty_allowed_in_usd + dec!("1"));
    cannot_initialize_package_royalty_if_greater_than_allowed(royalty_amount);
}

#[test]
fn cannot_initialize_component_royalty_if_greater_than_allowed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address =
        test_runner.compile_and_publish_with_owner("./tests/blueprints/royalty", owner_badge_addr);

    // Act
    let max_royalty_allowed = Decimal::try_from(DEFAULT_MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty",
                manifest_args!(max_royalty_allowed + dec!("1")),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ComponentRoyaltyError(
                ComponentRoyaltyError::RoyaltyAmountIsGreaterThanAllowed { .. }
            ))
        )
    });
}

#[test]
fn cannot_set_component_royalty_if_greater_than_allowed() {
    // Arrange
    let (
        mut test_runner,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();
    let max_royalty_allowed = Decimal::try_from(DEFAULT_MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                &btreeset!(NonFungibleLocalId::integer(1)),
            )
            .set_component_royalty(
                component_address,
                "paid_method",
                RoyaltyAmount::Xrd(max_royalty_allowed + dec!("1")),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ComponentRoyaltyError(
                ComponentRoyaltyError::RoyaltyAmountIsGreaterThanAllowed { .. }
            ))
        )
    });
}

#[test]
fn cannot_set_royalty_after_locking() {
    // Arrange
    let (
        mut test_runner,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                &btreeset!(NonFungibleLocalId::integer(1)),
            )
            .lock_component_royalty(component_address, "paid_method")
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                &btreeset!(NonFungibleLocalId::integer(1)),
            )
            .set_component_royalty(
                component_address,
                "paid_method".to_string(),
                RoyaltyAmount::Free,
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
        )
    });
}

fn set_up_package_and_component() -> (
    TestRunner,
    ComponentAddress,
    Secp256k1PublicKey,
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
            .lock_fee(account, 500u32.into())
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                &btreeset!(NonFungibleLocalId::integer(1)),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
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
