use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine_tests::common::*;
use radix_substate_store_queries::typed_substate_layout::{AuthZoneError, ComposeProofError};
use scrypto_test::prelude::*;

fn create_proof_internal(function_name: &str, error: Option<&str>) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof_creation"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ProofCreation",
            function_name,
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    if let Some(expected) = error {
        receipt.expect_commit_failure_containing_error(expected);
    } else {
        receipt.expect_commit_success();
    }
}

#[test]
fn can_create_proof_from_fungible_bucket() {
    create_proof_internal("create_proof_from_fungible_bucket_of_amount", None);
    create_proof_internal(
        "create_proof_from_fungible_bucket_of_non_fungibles",
        Some("Not a non-fungible bucket"),
    );
    create_proof_internal("create_proof_from_fungible_bucket_of_all", None);
}

#[test]
fn can_create_proof_from_non_fungible_bucket() {
    create_proof_internal(
        "create_proof_from_non_fungible_bucket_of_non_fungibles",
        None,
    );
    create_proof_internal("create_proof_from_non_fungible_bucket_of_all", None);
}

#[test]
fn can_create_proof_from_fungible_vault() {
    create_proof_internal("create_proof_from_fungible_vault_of_amount", None);
    create_proof_internal(
        "create_proof_from_fungible_vault_of_non_fungibles",
        Some("Not a non-fungible vault"),
    );
}

#[test]
fn can_create_proof_from_non_fungible_vault() {
    create_proof_internal("create_proof_from_non_fungible_vault", None);
    create_proof_internal(
        "create_proof_from_non_fungible_vault_of_non_fungibles",
        None,
    );
    create_proof_internal(
        "create_proof_from_non_fungible_vault_contains_non_fungible",
        None,
    );

    create_proof_internal(
        "create_proof_from_non_fungible_vault_of_empty_non_fungibles",
        Some("ProofError(EmptyProofNotAllowed)"),
    );
}

#[test]
fn can_create_proof_from_fungible_auth_zone() {
    create_proof_internal("create_proof_from_fungible_auth_zone_of_amount", None);
    create_proof_internal(
        "create_proof_from_fungible_auth_zone_of_non_fungibles",
        Some("NonFungibleOperationNotSupported"),
    );
    create_proof_internal("create_proof_from_fungible_auth_zone_of_all", None);
}

#[test]
fn can_create_proof_from_non_fungible_auth_zone() {
    create_proof_internal("create_proof_from_non_fungible_auth_zone_of_amount", None);
    create_proof_internal(
        "create_proof_from_non_fungible_auth_zone_of_non_fungibles",
        None,
    );
    create_proof_internal("create_proof_from_non_fungible_auth_zone_of_all", None);
}

#[test]
fn test_create_non_fungible_proof_with_large_amount() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _sk, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_auth_zone_of_amount(
            resource_address,
            dec!("1000000000000000000000000000000000000000"),
            "proof",
        )
        .drop_proof("proof")
        .drop_all_proofs()
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AuthZoneError(
                AuthZoneError::ComposeProofError(ComposeProofError::InvalidAmount)
            ))
        )
    })
}

fn compose_proof(amount: Decimal) -> u32 {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(false);
    let (pk2, _, account2) = ledger.new_account(false);
    let (pk3, _, account3) = ledger.new_account(false);
    let (pk4, _, account4) = ledger.new_account(false);

    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account1)
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .create_proof_from_account_of_amount(account3, XRD, 1)
        .create_proof_from_account_of_amount(account4, XRD, 1)
        .create_proof_from_auth_zone_of_amount(XRD, amount, "new_proof")
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![
            NonFungibleGlobalId::from_public_key(&pk1),
            NonFungibleGlobalId::from_public_key(&pk2),
            NonFungibleGlobalId::from_public_key(&pk3),
            NonFungibleGlobalId::from_public_key(&pk4),
        ],
    );
    receipt.fee_summary.total_execution_cost_units_consumed
}

#[test]
fn test_proof_composition() {
    let cost1 = compose_proof(dec!(1));
    let cost2 = compose_proof(dec!(2));
    let cost3 = compose_proof(dec!(3));

    let delta1 = (cost2 - cost1) as f32;
    let delta2 = (cost3 - cost2) as f32;
    // Assert that delta1 is roughly equal to delta2
    // The computation cost delta should be exactly equal, but there is substate
    // size difference, which affect many cost entries.
    assert!((delta2 - delta1).abs() / delta1 < 0.02);
}
