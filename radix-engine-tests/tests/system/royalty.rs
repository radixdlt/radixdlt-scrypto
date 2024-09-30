use radix_common::prelude::*;
use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::object_modules::royalty::ComponentRoyaltyError;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_component_royalty() {
    // Arrange
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.publish_package_simple(PackageLoader::get("royalty"));

    // Instantiate component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
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
    let account_pre_balance = ledger.get_component_balance(account, XRD);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
    assert_eq!(receipt.fee_summary.total_royalty_cost_in_xrd, dec!("3"));
    let account_post_balance = ledger.get_component_balance(account, XRD);
    let component_royalty = ledger.inspect_component_royalty(component_address).unwrap();
    assert_eq!(
        account_pre_balance
            .checked_sub(account_post_balance)
            .unwrap(),
        receipt.fee_summary.total_cost()
    );
    assert_eq!(
        component_royalty,
        receipt
            .fee_summary
            .total_royalty_cost_in_xrd
            .checked_sub(2)
            .unwrap()
    );
}

#[test]
fn test_component_royalty_in_usd() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.publish_package_simple(PackageLoader::get("royalty"));

    // Instantiate component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
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
    let account_pre_balance = ledger.get_component_balance(account, XRD);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(component_address, "paid_method_usd", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit(true);
    assert_eq!(
        receipt.fee_summary.total_royalty_cost_in_xrd,
        Decimal::try_from(USD_PRICE_IN_XRD)
            .unwrap()
            .checked_mul(Decimal::ONE)
            .unwrap()
    );
    let account_post_balance = ledger.get_component_balance(account, XRD);
    let component_royalty = ledger.inspect_component_royalty(component_address).unwrap();
    assert_eq!(
        account_pre_balance
            .checked_sub(account_post_balance)
            .unwrap(),
        receipt.fee_summary.total_cost()
    );
    assert_eq!(
        component_royalty,
        receipt.fee_summary.total_royalty_cost_in_xrd
    );
}

#[test]
fn test_package_royalty() {
    let (
        mut ledger,
        account,
        public_key,
        package_address,
        component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    let account_pre_balance = ledger.get_component_balance(account, XRD);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit(true);
    assert_eq!(
        receipt.fee_summary.total_royalty_cost_in_xrd,
        dec!(1).checked_add(dec!("2")).unwrap()
    );
    let account_post_balance = ledger.get_component_balance(account, XRD);
    let package_royalty = ledger.inspect_package_royalty(package_address).unwrap();
    let component_royalty = ledger.inspect_component_royalty(component_address).unwrap();
    assert_eq!(
        account_pre_balance
            .checked_sub(account_post_balance)
            .unwrap(),
        receipt.fee_summary.total_cost()
    );
    assert_eq!(package_royalty, dec!("2"));
    assert_eq!(component_royalty, dec!(1));
}

#[test]
fn test_royalty_accumulation_when_success() {
    let (
        mut ledger,
        account,
        public_key,
        package_address,
        component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit(true);
    assert_eq!(
        ledger.inspect_package_royalty(package_address),
        Some(dec!("2"))
    );
    assert_eq!(
        ledger.inspect_component_royalty(component_address).unwrap(),
        dec!(1)
    );
}

#[test]
fn test_royalty_accumulation_when_failure() {
    let (
        mut ledger,
        account,
        public_key,
        package_address,
        component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(component_address, "paid_method_panic", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_failure();
    assert_eq!(
        ledger.inspect_package_royalty(package_address),
        Some(Decimal::zero())
    );
    assert_eq!(
        ledger.inspect_component_royalty(component_address).unwrap(),
        Decimal::zero()
    );
}

#[test]
fn test_claim_royalty() {
    let (mut ledger, account, public_key, package_address, component_address, owner_badge_resource) =
        set_up_package_and_component();

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(component_address, "paid_method", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);
    receipt.expect_commit(true);
    assert_eq!(
        ledger.inspect_package_royalty(package_address),
        Some(dec!("2"))
    );
    assert_eq!(
        ledger.inspect_component_royalty(component_address).unwrap(),
        dec!(1)
    );

    // Claim package royalty
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .claim_package_royalties(package_address)
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

    // Claim component royalty
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .claim_component_royalties(component_address)
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

    // assert nothing left
    assert_eq!(
        ledger.inspect_package_royalty(package_address),
        Some(dec!("0"))
    );
    assert_eq!(
        ledger.inspect_component_royalty(component_address).unwrap(),
        dec!("0")
    );
}

fn cannot_initialize_package_royalty_if_greater_than_allowed(royalty_amount: RoyaltyAmount) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_public_key, _, account) = ledger.new_allocated_account();
    let owner_badge_resource = ledger.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));

    // Act
    let (code, mut definition) = PackageLoader::get("royalty");
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
        .lock_fee_from_faucet()
        .publish_package_with_owner(code, definition, owner_badge_addr)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let max_royalty_allowed_in_xrd = Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
    let royalty_amount =
        RoyaltyAmount::Xrd(max_royalty_allowed_in_xrd.checked_add(dec!(1)).unwrap());
    cannot_initialize_package_royalty_if_greater_than_allowed(royalty_amount);
}

#[test]
fn cannot_initialize_package_royalty_if_greater_usd_than_allowed() {
    let max_royalty_allowed_in_xrd = Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
    let max_royalty_allowed_in_usd = max_royalty_allowed_in_xrd
        .checked_div(Decimal::try_from(USD_PRICE_IN_XRD).unwrap())
        .unwrap();
    let royalty_amount =
        RoyaltyAmount::Usd(max_royalty_allowed_in_usd.checked_add(dec!(1)).unwrap());
    cannot_initialize_package_royalty_if_greater_than_allowed(royalty_amount);
}

#[test]
fn cannot_initialize_component_royalty_if_greater_than_allowed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let owner_badge_resource = ledger.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address =
        ledger.publish_package_with_owner(PackageLoader::get("royalty"), owner_badge_addr);

    // Act
    let max_royalty_allowed = Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty",
                manifest_args!(max_royalty_allowed.checked_add(dec!(1)).unwrap()),
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
        mut ledger,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();
    let max_royalty_allowed = Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .set_component_royalty(
                component_address,
                "paid_method",
                RoyaltyAmount::Xrd(max_royalty_allowed.checked_add(dec!(1)).unwrap()),
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
        mut ledger,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .lock_component_royalty(component_address, "paid_method")
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
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
            RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
        )
    });
}

fn set_up_package_and_component() -> (
    DefaultLedgerSimulator,
    ComponentAddress,
    Secp256k1PublicKey,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
) {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Publish package
    let owner_badge_resource = ledger.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address =
        ledger.publish_package_with_owner(PackageLoader::get("royalty"), owner_badge_addr);

    // Enable package royalty
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit(true);

    // Instantiate component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
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
        ledger,
        account,
        public_key,
        package_address,
        component_address,
        owner_badge_resource,
    )
}
