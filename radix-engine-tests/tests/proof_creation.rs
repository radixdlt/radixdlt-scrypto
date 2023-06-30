use radix_engine::{
    errors::{ApplicationError, RuntimeError},
    types::*,
};
use radix_engine_queries::typed_substate_layout::{AuthZoneError, ComposeProofError};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn create_proof_internal(function_name: &str, error: Option<&str>) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof_creation");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "ProofCreation",
            function_name,
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    if let Some(expected) = error {
        let error_message = receipt
            .expect_commit_failure()
            .outcome
            .expect_failure()
            .to_string();
        assert!(error_message.contains(expected))
    } else {
        receipt.expect_commit_success();
    }
}

#[test]
fn can_create_proof_from_fungible_bucket() {
    create_proof_internal("create_proof_from_fungible_bucket", None);
    create_proof_internal("create_proof_from_fungible_bucket_of_amount", None);
    create_proof_internal(
        "create_proof_from_fungible_bucket_of_non_fungibles",
        Some("assertion failed: self.resource_address().as_node_id().is_global_non_fungible_resource_manager()"),
    );
    create_proof_internal("create_proof_from_fungible_bucket_of_all", None);
}

#[test]
fn can_create_proof_from_non_fungible_bucket() {
    create_proof_internal("create_proof_from_non_fungible_bucket", None);
    create_proof_internal("create_proof_from_non_fungible_bucket_of_amount", None);
    create_proof_internal(
        "create_proof_from_non_fungible_bucket_of_non_fungibles",
        None,
    );
    create_proof_internal("create_proof_from_non_fungible_bucket_of_all", None);
}

#[test]
fn can_create_proof_from_fungible_vault() {
    create_proof_internal("create_proof_from_fungible_vault", None);
    create_proof_internal("create_proof_from_fungible_vault_of_amount", None);
    create_proof_internal(
        "create_proof_from_fungible_vault_of_non_fungibles",
        Some("assertion failed: self.0.as_node_id().is_internal_non_fungible_vault()"),
    );
}

#[test]
fn can_create_proof_from_non_fungible_vault() {
    create_proof_internal("create_proof_from_non_fungible_vault", None);
    create_proof_internal("create_proof_from_non_fungible_vault_of_amount", None);
    create_proof_internal(
        "create_proof_from_non_fungible_vault_of_non_fungibles",
        None,
    );
}

#[test]
fn can_create_proof_from_fungible_auth_zone() {
    create_proof_internal("create_proof_from_fungible_auth_zone", None);
    create_proof_internal("create_proof_from_fungible_auth_zone_of_amount", None);
    create_proof_internal(
        "create_proof_from_fungible_auth_zone_of_non_fungibles",
        Some("NonFungibleOperationNotSupported"),
    );
    create_proof_internal("create_proof_from_fungible_auth_zone_of_all", None);
}

#[test]
fn can_create_proof_from_non_fungible_auth_zone() {
    create_proof_internal("create_proof_from_non_fungible_auth_zone", None);
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
    let mut test_runner = TestRunner::builder().build();
    let (pk, _sk, account) = test_runner.new_account(false);
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_auth_zone_of_amount(
            resource_address,
            dec!("100000000000000000000000000000000000000000000"),
            |builder, proof| builder.drop_proof(proof),
        )
        .drop_all_proofs()
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

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
