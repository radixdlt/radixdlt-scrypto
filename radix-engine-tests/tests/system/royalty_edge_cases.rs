use radix_engine::errors::*;
use radix_engine::transaction::*;
use radix_engine_tests::common::*;
use radix_substate_store_queries::typed_substate_layout::*;
use scrypto_test::prelude::*;

const DECIMAL_MIN: Decimal = Decimal::MIN;
const DECIMAL_MAX: Decimal = Decimal::MAX;

const DECIMAL_ZERO: Decimal = Decimal::ZERO;
const DECIMAL_VERY_SMALL: Decimal = Decimal::ONE_ATTO;

const DECIMAL_ONE: Decimal = Decimal::ONE;

lazy_static::lazy_static! {
    static ref CODE_AND_DEF: (Vec<u8>, PackageDefinition) = PackageLoader::get("royalty-edge-cases");
}

component_instantiation_tests! {
    instantiation_of_component_with_decimal_min_xrd_royalties_fails(
        RoyaltyAmount::Xrd(DECIMAL_MIN),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    instantiation_of_component_with_decimal_min_usd_royalties_fails(
        RoyaltyAmount::Usd(DECIMAL_MIN),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    instantiation_of_component_with_negative_one_xrd_royalties_fails(
        RoyaltyAmount::Xrd(-DECIMAL_ONE),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    instantiation_of_component_with_negative_one_usd_royalties_fails(
        RoyaltyAmount::Usd(-DECIMAL_ONE),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    instantiation_of_component_with_very_small_decimal_xrd_royalties_fails(
        RoyaltyAmount::Xrd(-DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    instantiation_of_component_with_very_small_decimal_usd_royalties_fails(
        RoyaltyAmount::Usd(-DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    instantiation_of_component_with_zero_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(DECIMAL_ZERO),
        None
    );

    instantiation_of_component_with_zero_usd_royalties_succeeds(
        RoyaltyAmount::Usd(DECIMAL_ZERO),
        None
    );

    instantiation_of_component_with_free_royalties_succeeds(
        RoyaltyAmount::Free,
        None
    );

    instantiation_of_component_with_very_small_positive_amount_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(DECIMAL_VERY_SMALL),
        None
    );

    instantiation_of_component_with_very_small_positive_amount_usd_royalties_succeeds(
        RoyaltyAmount::Usd(DECIMAL_VERY_SMALL),
        None
    );

    instantiation_of_component_with_one_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(DECIMAL_ONE),
        None
    );

    instantiation_of_component_with_one_usd_royalties_succeeds(
        RoyaltyAmount::Usd(DECIMAL_ONE),
        None
    );

    instantiation_of_component_with_slightly_less_than_maximum_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() - DECIMAL_VERY_SMALL),
        None,
    );

    instantiation_of_component_with_slightly_less_than_maximum_usd_royalties_succeeds(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() - DECIMAL_VERY_SMALL),
        None,
    );

    instantiation_of_component_with_maximum_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd()),
        None,
    );

    instantiation_of_component_with_maximum_usd_royalties_succeeds(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd()),
        None,
    );

    instantiation_of_component_with_slightly_more_than_maximum_xrd_royalties_fails(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() + DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    instantiation_of_component_with_slightly_more_than_maximum_usd_royalties_fails(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() + DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    instantiation_of_component_with_decimal_max_xrd_royalties_fails(
        RoyaltyAmount::Xrd(DECIMAL_MAX),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    instantiation_of_component_with_decimal_max_usd_royalties_fails(
        RoyaltyAmount::Usd(DECIMAL_MAX),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );
}

component_interaction_tests! {
    interactions_with_component_with_decimal_min_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_MIN),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_component_with_decimal_min_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_MIN),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_component_with_negative_one_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(-DECIMAL_ONE),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_component_with_negative_one_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(-DECIMAL_ONE),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_component_with_very_small_decimal_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(-DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_component_with_very_small_decimal_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(-DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_component_with_zero_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_ZERO),
        None
    );

    interactions_with_component_with_zero_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_ZERO),
        None
    );

    interactions_with_component_with_free_royalties_proceeds_as_expected(
        RoyaltyAmount::Free,
        None
    );

    interactions_with_component_with_very_small_positive_amount_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_VERY_SMALL),
        None
    );

    interactions_with_component_with_very_small_positive_amount_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_VERY_SMALL),
        None
    );

    interactions_with_component_with_one_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_ONE),
        None
    );

    interactions_with_component_with_one_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_ONE),
        None
    );

    interactions_with_component_with_slightly_less_than_maximum_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() - DECIMAL_VERY_SMALL),
        None,
    );

    interactions_with_component_with_slightly_less_than_maximum_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() - DECIMAL_VERY_SMALL),
        None,
    );

    interactions_with_component_with_maximum_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd()),
        None,
    );

    interactions_with_component_with_maximum_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd()),
        None,
    );

    interactions_with_component_with_slightly_more_than_maximum_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() + DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    interactions_with_component_with_slightly_more_than_maximum_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() + DECIMAL_VERY_SMALL),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    interactions_with_component_with_decimal_max_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_MAX),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    interactions_with_component_with_decimal_max_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_MAX),
        Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
    );
}

package_publishing_tests! {
    publishing_of_package_with_decimal_min_xrd_royalties_fails(
        RoyaltyAmount::Xrd(DECIMAL_MIN),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    publishing_of_package_with_decimal_min_usd_royalties_fails(
        RoyaltyAmount::Usd(DECIMAL_MIN),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    publishing_of_package_with_negative_one_xrd_royalties_fails(
        RoyaltyAmount::Xrd(-DECIMAL_ONE),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    publishing_of_package_with_negative_one_usd_royalties_fails(
        RoyaltyAmount::Usd(-DECIMAL_ONE),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    publishing_of_package_with_very_small_decimal_xrd_royalties_fails(
        RoyaltyAmount::Xrd(-DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    publishing_of_package_with_very_small_decimal_usd_royalties_fails(
        RoyaltyAmount::Usd(-DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    publishing_of_package_with_zero_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(DECIMAL_ZERO),
        None
    );

    publishing_of_package_with_zero_usd_royalties_succeeds(
        RoyaltyAmount::Usd(DECIMAL_ZERO),
        None
    );

    publishing_of_package_with_free_royalties_succeeds(
        RoyaltyAmount::Free,
        None
    );

    publishing_of_package_with_very_small_positive_amount_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(DECIMAL_VERY_SMALL),
        None
    );

    publishing_of_package_with_very_small_positive_amount_usd_royalties_succeeds(
        RoyaltyAmount::Usd(DECIMAL_VERY_SMALL),
        None
    );

    publishing_of_package_with_one_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(DECIMAL_ONE),
        None
    );

    publishing_of_package_with_one_usd_royalties_succeeds(
        RoyaltyAmount::Usd(DECIMAL_ONE),
        None
    );

    publishing_of_package_with_slightly_less_than_maximum_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() - DECIMAL_VERY_SMALL),
        None,
    );

    publishing_of_package_with_slightly_less_than_maximum_usd_royalties_succeeds(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() - DECIMAL_VERY_SMALL),
        None,
    );

    publishing_of_package_with_maximum_xrd_royalties_succeeds(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd()),
        None,
    );

    publishing_of_package_with_maximum_usd_royalties_succeeds(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd()),
        None,
    );

    publishing_of_package_with_slightly_more_than_maximum_xrd_royalties_fails(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() + DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    publishing_of_package_with_slightly_more_than_maximum_usd_royalties_fails(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() + DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    publishing_of_package_with_decimal_max_xrd_royalties_fails(
        RoyaltyAmount::Xrd(DECIMAL_MAX),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    publishing_of_package_with_decimal_max_usd_royalties_fails(
        RoyaltyAmount::Usd(DECIMAL_MAX),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );
}

package_interactions_tests! {
    interactions_with_package_with_decimal_min_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_MIN),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_package_with_decimal_min_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_MIN),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_package_with_negative_one_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(-DECIMAL_ONE),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_package_with_negative_one_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(-DECIMAL_ONE),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_package_with_very_small_decimal_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(-DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_package_with_very_small_decimal_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(-DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_negative),
    );

    interactions_with_package_with_zero_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_ZERO),
        None
    );

    interactions_with_package_with_zero_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_ZERO),
        None
    );

    interactions_with_package_with_free_royalties_proceeds_as_expected(
        RoyaltyAmount::Free,
        None
    );

    interactions_with_package_with_very_small_positive_amount_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_VERY_SMALL),
        None
    );

    interactions_with_package_with_very_small_positive_amount_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_VERY_SMALL),
        None
    );

    interactions_with_package_with_one_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_ONE),
        None
    );

    interactions_with_package_with_one_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_ONE),
        None
    );

    interactions_with_package_with_slightly_less_than_maximum_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() - DECIMAL_VERY_SMALL),
        None,
    );

    interactions_with_package_with_slightly_less_than_maximum_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() - DECIMAL_VERY_SMALL),
        None,
    );

    interactions_with_package_with_maximum_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd()),
        None,
    );

    interactions_with_package_with_maximum_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd()),
        None,
    );

    interactions_with_package_with_slightly_more_than_maximum_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd() + DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    interactions_with_package_with_slightly_more_than_maximum_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(max_per_function_royalty_in_usd() + DECIMAL_VERY_SMALL),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    interactions_with_package_with_decimal_max_xrd_royalties_proceeds_as_expected(
        RoyaltyAmount::Xrd(DECIMAL_MAX),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );

    interactions_with_package_with_decimal_max_usd_royalties_proceeds_as_expected(
        RoyaltyAmount::Usd(DECIMAL_MAX),
        Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
    );
}

#[test]
fn test_package_with_non_exhaustive_package_royalties_fails_instantiation() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = CODE_AND_DEF.clone();

    for blueprint_definition in definition.blueprints.values_mut() {
        blueprint_definition.royalty_config = PackageRoyaltyConfig::Enabled(Default::default())
    }

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition.clone(),
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::UnexpectedNumberOfFunctionRoyalties { .. }
            ))
        )
    });
}

#[test]
fn component_and_package_royalties_are_both_applied() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = CODE_AND_DEF.clone();
    let royalty_amount = RoyaltyAmount::Xrd(10.into());
    update_package_royalties(&mut definition, royalty_amount);

    let package_address = ledger.publish_package_simple((code, definition));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoyaltyEdgeCases",
            "instantiate",
            manifest_args!(royalty_amount),
        )
        .build();
    let component_address = *ledger
        .execute_manifest(manifest, vec![])
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "method", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(receipt.fee_summary.total_royalty_cost_in_xrd, dec!("20"));
}

/// This test is here to check if the following bit of code can cause a panic in the Royalty module:
/// https://github.com/radixdlt/radixdlt-scrypto/blob/v0.12.1/radix-engine/src/system/node_modules/royalty/package.rs#L455
/// I suspected that this could cause a panic since we're defaulting to one type and then calling
/// `.as_typed` with a different type later down in the code. It turns out that this bit doesn't
/// panic since all [`KeyValueEntrySubstate::<T>::default()`] are equal when we scrypto encode them,
/// so decoding as a different type is no issue. I've still went ahead and changed the type there as
/// I believe that it's better.
#[test]
fn test_component_with_missing_method_royalty() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(CODE_AND_DEF.clone());

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoyaltyEdgeCases",
            "instantiate_with_missing_method_royalty",
            manifest_args!(),
        )
        .build();
    let component_address = *ledger
        .execute_manifest(manifest, vec![])
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "method", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(receipt.fee_summary.total_royalty_cost_in_xrd, dec!("0"))
}

fn royalty_amount_to_xrd(royalty_amount: &RoyaltyAmount) -> Decimal {
    match royalty_amount {
        RoyaltyAmount::Free => Decimal::zero(),
        RoyaltyAmount::Xrd(amount) => *amount,
        RoyaltyAmount::Usd(amount) => *amount * CostingParameters::babylon_genesis().usd_price,
    }
}

fn is_component_royalty_error_royalty_amount_is_negative(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::ComponentRoyaltyError(
            ComponentRoyaltyError::RoyaltyAmountIsNegative(..)
        ))
    )
}

fn is_component_royalty_error_royalty_amount_is_greater_than_allowed(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::ComponentRoyaltyError(
            ComponentRoyaltyError::RoyaltyAmountIsGreaterThanAllowed { .. },
        ),)
    )
}

fn is_package_royalty_error_royalty_amount_is_negative(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::PackageError(
            PackageError::RoyaltyAmountIsNegative(..)
        ))
    )
}

fn is_package_royalty_error_royalty_amount_is_greater_than_allowed(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::PackageError(
            PackageError::RoyaltyAmountIsGreaterThanAllowed { .. },
        ),)
    )
}

fn update_package_royalties(
    package_definition: &mut PackageDefinition,
    royalty_amount: RoyaltyAmount,
) {
    for blueprint_definition in package_definition.blueprints.values_mut() {
        let function_royalties = blueprint_definition
            .schema
            .functions
            .functions
            .keys()
            .map(|key| (key.clone(), royalty_amount))
            .collect::<IndexMap<_, _>>();
        blueprint_definition.royalty_config = PackageRoyaltyConfig::Enabled(function_royalties)
    }
}

fn max_per_function_royalty_in_xrd() -> Decimal {
    Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap()
}

fn max_per_function_royalty_in_usd() -> Decimal {
    let max_per_function_royalty_in_xrd = max_per_function_royalty_in_xrd();
    let CostingParameters { usd_price, .. } = CostingParameters::babylon_genesis();
    max_per_function_royalty_in_xrd / usd_price
}

macro_rules! component_instantiation_tests {
    (
        $(
            $name: ident (
                $royalty_amount: expr,
                $error_checking_fn: expr $(,)?
            );
        )*
    ) => {
        $(
            #[test]
            fn $name() {
                // Arrange
                let royalty_amount: RoyaltyAmount = $royalty_amount;
                let error_checking_fn: Option<fn(&RuntimeError) -> bool> = $error_checking_fn;

                let mut ledger = LedgerSimulatorBuilder::new().build();
                let package_address =
                    ledger.publish_package_simple(CODE_AND_DEF.clone());

                // Act
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        package_address,
                        "RoyaltyEdgeCases",
                        "instantiate",
                        manifest_args!(royalty_amount),
                    )
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);

                // Assert
                match error_checking_fn {
                    Some(func) => {
                        receipt.expect_specific_failure(func);
                    }
                    None => {
                        receipt.expect_commit_success();
                    }
                };
            }
        )*
    };
}

macro_rules! component_interaction_tests {
    (
        $(
            $name: ident (
                $royalty_amount: expr,
                $error_checking_fn: expr $(,)?
            );
        )*
    ) => {
        $(
            #[test]
            fn $name() {
                // Arrange
                let royalty_amount: RoyaltyAmount = $royalty_amount;
                let error_checking_fn: Option<fn(&RuntimeError) -> bool> = $error_checking_fn;

                let mut ledger = LedgerSimulatorBuilder::new().build();
                let package_address =
                    ledger.publish_package_simple(CODE_AND_DEF.clone());

                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        package_address,
                        "RoyaltyEdgeCases",
                        "instantiate",
                        manifest_args!(royalty_amount),
                    )
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);

                let commit_result = match error_checking_fn {
                    Some(func) => {
                        receipt.expect_specific_failure(func);
                        return; /* If component instantiation failed, that's correct behavior, exit early */
                    }
                    None => {
                        receipt.expect_commit_success();
                        receipt.expect_commit_success()
                    }
                };
                let component_address = *commit_result.new_component_addresses().first().unwrap();

                // Act
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(component_address, "method", manifest_args!())
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);

                // Assert
                assert_eq!(
                    receipt.fee_summary.total_royalty_cost_in_xrd,
                    royalty_amount_to_xrd(&royalty_amount)
                )
            }
        )*
    };
}

macro_rules! package_publishing_tests {
    (
        $(
            $name: ident (
                $royalty_amount: expr,
                $error_checking_fn: expr $(,)?
            );
        )*
    ) => {
        $(
            #[test]
            fn $name() {
                // Arrange
                let royalty_amount: RoyaltyAmount = $royalty_amount;
                let error_checking_fn: Option<fn(&RuntimeError) -> bool> = $error_checking_fn;

                let mut ledger = LedgerSimulatorBuilder::new().build();
                let (code, mut definition) = CODE_AND_DEF.clone();

                update_package_royalties(&mut definition, royalty_amount);

                // Act
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .publish_package_advanced(
                        None,
                        code.clone(),
                        definition.clone(),
                        MetadataInit::default(),
                        OwnerRole::None,
                    )
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);

                // Assert
                match error_checking_fn {
                    Some(func) => {
                        receipt.expect_specific_failure(func);
                    }
                    None => {
                        receipt.expect_commit_success();
                    }
                };
            }
        )*
    };
}

macro_rules! package_interactions_tests {
    (
        $(
            $name: ident (
                $royalty_amount: expr,
                $error_checking_fn: expr $(,)?
            );
        )*
    ) => {
        $(
            #[test]
            fn $name() {
                // Arrange
                let royalty_amount: RoyaltyAmount = $royalty_amount;
                let error_checking_fn: Option<fn(&RuntimeError) -> bool> = $error_checking_fn;

                let mut ledger = LedgerSimulatorBuilder::new().build();
                let (code, mut definition) = CODE_AND_DEF.clone();

                update_package_royalties(&mut definition, royalty_amount);

                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .publish_package_advanced(
                        None,
                        code.clone(),
                        definition.clone(),
                        MetadataInit::default(),
                        OwnerRole::None,
                    )
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);

                let commit_result = match error_checking_fn {
                    Some(func) => {
                        receipt.expect_specific_failure(func);
                        return; /* If component instantiation failed, that's correct behavior, exit early */
                    }
                    None => {
                        receipt.expect_commit_success();
                        receipt.expect_commit_success()
                    }
                };
                let package_address = *commit_result.new_package_addresses().first().unwrap();

                // Act
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        package_address,
                        "RoyaltyEdgeCases",
                        "func",
                        manifest_args!(),
                    )
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);

                // Assert
                assert_eq!(
                    receipt.fee_summary.total_royalty_cost_in_xrd,
                    royalty_amount_to_xrd(&royalty_amount)
                )
            }
        )*
    };
}
use component_instantiation_tests;
use component_interaction_tests;
use package_interactions_tests;
use package_publishing_tests;
