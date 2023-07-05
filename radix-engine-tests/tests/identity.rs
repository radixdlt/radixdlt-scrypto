use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::BalanceChange;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::blueprints::identity::{
    IdentityCreateAdvancedInput, IdentitySecurifyToSingleBadgeInput, IDENTITY_BLUEPRINT,
    IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_SECURIFY_IDENT,
};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn cannot_securify_in_advanced_mode() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);
    let component_address = test_runner.new_identity(pk.clone(), false);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            to_manifest_value_and_unwrap!(&IdentitySecurifyToSingleBadgeInput {}),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                AuthError::Unauthorized { .. }
            ))
        )
    });
}

#[test]
fn can_securify_from_virtual_identity() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);
    let component_address = test_runner.new_identity(pk.clone(), true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            to_manifest_value_and_unwrap!(&IdentitySecurifyToSingleBadgeInput {}),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_securify_twice() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);
    let component_address = test_runner.new_identity(pk.clone(), true);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            to_manifest_value_and_unwrap!(&IdentitySecurifyToSingleBadgeInput {}),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            to_manifest_value_and_unwrap!(&IdentitySecurifyToSingleBadgeInput {}),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                AuthError::Unauthorized { .. }
            ))
        )
    });
}

#[test]
fn can_set_metadata_after_securify() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);
    let identity_address = test_runner.new_identity(pk.clone(), true);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            identity_address,
            IDENTITY_SECURIFY_IDENT,
            to_manifest_value_and_unwrap!(&IdentitySecurifyToSingleBadgeInput {}),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account(account, IDENTITY_OWNER_BADGE)
        .set_metadata(
            identity_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(identity_address.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataValue::String("best package ever!".to_string())
    );
}

#[test]
fn can_set_metadata_on_securified_identity() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);
    let identity_address = test_runner.new_securified_identity(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account(account, IDENTITY_OWNER_BADGE)
        .set_metadata(
            identity_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(identity_address.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataValue::String("best package ever!".to_string())
    );
}

#[test]
fn securified_identity_is_owned_by_correct_owner_badge() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let pk = Secp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let identity = test_runner.new_identity(pk, true);
    let (_, _, account) = test_runner.new_account(true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            identity,
            IDENTITY_SECURIFY_IDENT,
            to_manifest_value_and_unwrap!(&IdentitySecurifyToSingleBadgeInput {}),
        )
        .try_deposit_batch_or_refund(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    let balance_changes = receipt.expect_commit_success().balance_changes();
    let balance_change = balance_changes
        .get(&GlobalAddress::from(account))
        .unwrap()
        .get(&IDENTITY_OWNER_BADGE)
        .unwrap()
        .clone();
    assert_eq!(
        balance_change,
        BalanceChange::NonFungible {
            added: btreeset![NonFungibleLocalId::bytes(identity.as_node_id().0).unwrap()],
            removed: btreeset![]
        }
    )
}

#[test]
fn identity_created_with_create_advanced_has_an_empty_owner_badge() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let identity = {
        let manifest = ManifestBuilder::new()
            .call_function(
                IDENTITY_PACKAGE,
                IDENTITY_BLUEPRINT,
                IDENTITY_CREATE_ADVANCED_IDENT,
                to_manifest_value(&IdentityCreateAdvancedInput {
                    owner_rule: OwnerRole::None,
                })
                .unwrap(),
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    // Act
    let metadata = test_runner.get_metadata(identity.into(), "owner_badge");

    // Assert
    assert!(is_metadata_empty(&metadata))
}

fn is_metadata_empty(metadata_value: &Option<MetadataValue>) -> bool {
    if let None = metadata_value {
        true
    } else {
        false
    }
}
