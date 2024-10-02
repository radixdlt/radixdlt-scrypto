use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_db_reader::SystemReaderError;
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::BalanceChange;
use radix_engine_interface::blueprints::identity::{
    IdentityCreateAdvancedInput, IdentitySecurifyToSingleBadgeInput, IDENTITY_BLUEPRINT,
    IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_SECURIFY_IDENT,
};
use radix_engine_interface::object_modules::metadata::MetadataValue;
use radix_engine_tests::common::PackageLoader;
use scrypto_test::prelude::*;

#[test]
fn cannot_securify_in_advanced_mode() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let component_address = ledger.new_identity(pk.clone(), false);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            IdentitySecurifyToSingleBadgeInput {},
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let component_address = ledger.new_identity(pk.clone(), true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            IdentitySecurifyToSingleBadgeInput {},
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_securify_from_virtual_identity_ed25519() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_ed25519_preallocated_account();
    let component_address = ledger.new_identity(pk.clone(), true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            IdentitySecurifyToSingleBadgeInput {},
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_securify_twice() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let component_address = ledger.new_identity(pk.clone(), true);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            IdentitySecurifyToSingleBadgeInput {},
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            IDENTITY_SECURIFY_IDENT,
            IdentitySecurifyToSingleBadgeInput {},
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let identity_address = ledger.new_identity(pk.clone(), true);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            identity_address,
            IDENTITY_SECURIFY_IDENT,
            IdentitySecurifyToSingleBadgeInput {},
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            IDENTITY_OWNER_BADGE,
            [NonFungibleLocalId::bytes(identity_address.as_node_id().0).unwrap()],
        )
        .set_metadata(
            identity_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
    let value = ledger
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let identity_address = ledger.new_securified_identity(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            IDENTITY_OWNER_BADGE,
            [NonFungibleLocalId::bytes(identity_address.as_node_id().0).unwrap()],
        )
        .set_metadata(
            identity_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
    let value = ledger
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let pk = Secp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let identity = ledger.new_identity(pk, true);
    let (_, _, account) = ledger.new_account(true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            identity,
            IDENTITY_SECURIFY_IDENT,
            IdentitySecurifyToSingleBadgeInput {},
        )
        .try_deposit_entire_worktop_or_refund(account, None)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    let balance_change = ledger
        .sum_descendant_balance_changes(receipt.expect_commit_success(), account.as_node_id())
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let identity = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                IDENTITY_PACKAGE,
                IDENTITY_BLUEPRINT,
                IDENTITY_CREATE_ADVANCED_IDENT,
                IdentityCreateAdvancedInput {
                    owner_role: OwnerRole::None,
                },
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };

    // Act
    let metadata = ledger.get_metadata(identity.into(), "owner_badge");

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

#[test]
fn identity_created_before_cuttlefish_has_royalty_module() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Bottlenose))
        .build();
    let (pk, _, account) = ledger.new_account(false);
    let package_address = ledger.publish_package_simple(PackageLoader::get("identity"));

    for (package, blueprint, id, args) in [
        (
            IDENTITY_PACKAGE,
            IDENTITY_BLUEPRINT,
            IDENTITY_CREATE_IDENT,
            manifest_args!(),
        ),
        (
            IDENTITY_PACKAGE,
            IDENTITY_BLUEPRINT,
            IDENTITY_CREATE_ADVANCED_IDENT,
            manifest_args!(OwnerRole::None),
        ),
        (
            package_address,
            "IdentityTest",
            "accept_address",
            manifest_args!(ComponentAddress::new_or_panic(
                [EntityType::GlobalPreallocatedEd25519Identity as u8; NodeId::LENGTH]
            )),
        ),
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package, blueprint, id, args)
            .deposit_entire_worktop(account)
            .build();
        let identity = ledger
            .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)])
            .expect_commit_success()
            .new_component_addresses()[0];

        // Act
        let royalty = ledger.inspect_component_royalty(identity);

        // Assert
        assert_eq!(royalty, Ok(dec!(0)));
    }
}

#[test]
fn identity_created_after_cuttlefish_has_no_royalty_module() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let package_address = ledger.publish_package_simple(PackageLoader::get("identity"));

    for (package, blueprint, id, args) in [
        (
            IDENTITY_PACKAGE,
            IDENTITY_BLUEPRINT,
            IDENTITY_CREATE_IDENT,
            manifest_args!(),
        ),
        (
            IDENTITY_PACKAGE,
            IDENTITY_BLUEPRINT,
            IDENTITY_CREATE_ADVANCED_IDENT,
            manifest_args!(OwnerRole::None),
        ),
        (
            package_address,
            "IdentityTest",
            "accept_address",
            manifest_args!(ComponentAddress::new_or_panic(
                [EntityType::GlobalPreallocatedEd25519Identity as u8; NodeId::LENGTH]
            )),
        ),
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package, blueprint, id, args)
            .deposit_entire_worktop(account)
            .build();
        let identity = ledger
            .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)])
            .expect_commit_success()
            .new_component_addresses()[0];

        // Act
        let royalty = ledger.inspect_component_royalty(identity);

        // Assert
        assert_eq!(royalty, Err(SystemReaderError::ModuleDoesNotExist));
    }
}
