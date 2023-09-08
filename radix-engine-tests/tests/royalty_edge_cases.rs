mod package_loader;

use package_loader::PackageLoader;
use radix_engine::errors::*;
use radix_engine::transaction::*;
use radix_engine_queries::typed_substate_layout::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_component_royalty_edge_cases_at_instantiation() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let package_address =
        test_runner.publish_package_simple(PackageLoader::get("royalty-edge-cases"));

    for (royalty_amount, error_checking_fn, _) in test_cases().into_iter() {
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
        let receipt = test_runner.execute_manifest(manifest, vec![]);

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
}

#[test]
fn test_package_royalty_edge_cases_at_publishing() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let (code, mut definition) = PackageLoader::get("royalty-edge-cases");

    for (royalty_amount, _, error_checking_fn) in test_cases().into_iter() {
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
        let receipt = test_runner.execute_manifest(manifest, vec![]);

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
}

#[test]
fn test_component_royalty_edge_cases_at_interactions() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let package_address =
        test_runner.publish_package_simple(PackageLoader::get("royalty-edge-cases"));

    for (royalty_amount, error_checking_fn, _) in test_cases().into_iter() {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "RoyaltyEdgeCases",
                "instantiate",
                manifest_args!(royalty_amount),
            )
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);

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
        let receipt = test_runner.execute_manifest(manifest, vec![]);

        // Assert
        assert_eq!(
            receipt.fee_summary.total_royalty_cost_in_xrd,
            royalty_amount_to_xrd(&royalty_amount)
        )
    }
}

#[test]
fn test_package_royalty_edge_cases_at_interactions() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let (code, mut definition) = PackageLoader::get("royalty-edge-cases");

    for (royalty_amount, _, error_checking_fn) in test_cases().into_iter() {
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
        let receipt = test_runner.execute_manifest(manifest, vec![]);

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
        let receipt = test_runner.execute_manifest(manifest, vec![]);

        // Assert
        assert_eq!(
            receipt.fee_summary.total_royalty_cost_in_xrd,
            royalty_amount_to_xrd(&royalty_amount)
        )
    }
}

fn royalty_amount_to_xrd(royalty_amount: &RoyaltyAmount) -> Decimal {
    match royalty_amount {
        RoyaltyAmount::Free => Decimal::zero(),
        RoyaltyAmount::Xrd(amount) => *amount,
        RoyaltyAmount::Usd(amount) => *amount / CostingParameters::default().usd_price,
    }
}

fn test_cases() -> Vec<(
    RoyaltyAmount,
    Option<fn(&RuntimeError) -> bool>,
    Option<fn(&RuntimeError) -> bool>,
)> {
    let network_definition = NetworkDefinition::simulator();
    let ExecutionConfig {
        max_per_function_royalty_in_xrd,
        ..
    } = ExecutionConfig::for_notarized_transaction(network_definition);
    let CostingParameters { usd_price, .. } = CostingParameters::default();

    let max_per_function_royalty_in_usd = max_per_function_royalty_in_xrd / usd_price;
    let very_small_decimal = dec!("0.000000000000000001");

    vec![
        /* Negative royalty amounts are not permitted */
        (
            RoyaltyAmount::Xrd(Decimal::MIN),
            Some(is_component_royalty_error_royalty_amount_is_negative),
            Some(is_package_royalty_error_royalty_amount_is_negative),
        ),
        (
            RoyaltyAmount::Usd(Decimal::MIN),
            Some(is_component_royalty_error_royalty_amount_is_negative),
            Some(is_package_royalty_error_royalty_amount_is_negative),
        ),
        (
            RoyaltyAmount::Xrd(dec!("-1")),
            Some(is_component_royalty_error_royalty_amount_is_negative),
            Some(is_package_royalty_error_royalty_amount_is_negative),
        ),
        (
            RoyaltyAmount::Usd(dec!("-1")),
            Some(is_component_royalty_error_royalty_amount_is_negative),
            Some(is_package_royalty_error_royalty_amount_is_negative),
        ),
        (
            RoyaltyAmount::Xrd(-very_small_decimal),
            Some(is_component_royalty_error_royalty_amount_is_negative),
            Some(is_package_royalty_error_royalty_amount_is_negative),
        ),
        (
            RoyaltyAmount::Usd(-very_small_decimal),
            Some(is_component_royalty_error_royalty_amount_is_negative),
            Some(is_package_royalty_error_royalty_amount_is_negative),
        ),
        /* Zero is permitted */
        (RoyaltyAmount::Free, None, None),
        (RoyaltyAmount::Xrd(Decimal::ZERO), None, None),
        (RoyaltyAmount::Usd(Decimal::ZERO), None, None),
        /* Positive Less than Max is permitted */
        (RoyaltyAmount::Xrd(very_small_decimal), None, None),
        (RoyaltyAmount::Usd(very_small_decimal), None, None),
        (RoyaltyAmount::Xrd(dec!("1")), None, None),
        (RoyaltyAmount::Usd(dec!("1")), None, None),
        (
            RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd - very_small_decimal),
            None,
            None,
        ),
        (
            RoyaltyAmount::Usd(max_per_function_royalty_in_usd - very_small_decimal),
            None,
            None,
        ),
        /* Maximum of XRD and USD is permitted */
        (
            RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd),
            None,
            None,
        ),
        (
            RoyaltyAmount::Usd(max_per_function_royalty_in_usd),
            None,
            None,
        ),
        /* Anything above maximum is not permitted */
        (
            RoyaltyAmount::Xrd(max_per_function_royalty_in_xrd + very_small_decimal),
            Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
            Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
        ),
        (
            RoyaltyAmount::Usd(max_per_function_royalty_in_usd + very_small_decimal),
            Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
            Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
        ),
        (
            RoyaltyAmount::Xrd(Decimal::MAX),
            Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
            Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
        ),
        (
            RoyaltyAmount::Usd(Decimal::MAX),
            Some(is_component_royalty_error_royalty_amount_is_greater_than_allowed),
            Some(is_package_royalty_error_royalty_amount_is_greater_than_allowed),
        ),
    ]
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
