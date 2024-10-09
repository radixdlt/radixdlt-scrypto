use radix_common::prelude::*;
use radix_engine::blueprints::consensus_manager::{
    ClaimXrdEvent, EpochChangeEvent, RegisterValidatorEvent, RoundChangeEvent, StakeEvent,
    UnregisterValidatorEvent, UnstakeEvent, UpdateAcceptingStakeDelegationStateEvent,
};
use radix_engine::blueprints::package::PackageError;
use radix_engine::blueprints::{account, resource::*};
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::object_modules::metadata::SetMetadataEvent;
use radix_engine::system::system_type_checker::TypeCheckError;
use radix_engine::updates::BabylonSettings;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::account::ResourcePreference;
use radix_engine_interface::blueprints::account::ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::package::BlueprintPayloadIdentifier;
use radix_engine_interface::object_modules::metadata::MetadataValue;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::{burn_roles, metadata, metadata_init, mint_roles, recall_roles};
use radix_engine_tests::common::*;
use scrypto::prelude::{AccessRule, FromPublicKey};
use scrypto::NonFungibleData;
use scrypto_test::prelude::*;

#[test]
fn test_events_of_commit_failure() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, dec!(100))
        .withdraw_from_account(account, XRD, dec!(100)) // reverted
        .assert_worktop_contains(XRD, dec!(500))
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    let events = &receipt.expect_commit_failure().application_events;
    for event in events {
        let name = ledger.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
    }
    assert_eq!(events.len(), 4);
    assert!(match events.get(0) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
            && is_decoded_equal(
                &fungible_vault::LockFeeEvent { amount: 100.into() },
                event_data
            ) =>
            true,
        _ => false,
    });
    assert!(match events.get(1) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<fungible_vault::PayFeeEvent>(event_identifier)
            && is_decoded_equal(
                &fungible_vault::PayFeeEvent {
                    amount: receipt.fee_summary.total_cost()
                },
                event_data
            ) =>
            true,
        _ => false,
    });
    assert!(match events.get(2) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier)
            && is_decoded_equal(
                &fungible_vault::DepositEvent {
                    amount: receipt
                        .expect_commit_failure()
                        .fee_destination
                        .to_proposer
                        .checked_add(
                            receipt
                                .expect_commit_failure()
                                .fee_destination
                                .to_validator_set
                        )
                        .unwrap()
                },
                event_data
            ) =>
            true,
        _ => false,
    });
    assert!(match events.get(3) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<BurnFungibleResourceEvent>(event_identifier)
            && is_decoded_equal(
                &BurnFungibleResourceEvent {
                    amount: receipt.expect_commit_failure().fee_destination.to_burn
                },
                event_data
            ) =>
            true,
        _ => false,
    });
}

#[test]
fn create_proof_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, dec!(500))
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    let events = &receipt.expect_commit_success().application_events;
    for event in events {
        let name = ledger.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
    }
    assert_eq!(events.len(), 4);
    assert!(match events.get(0) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
            && is_decoded_equal(
                &fungible_vault::LockFeeEvent { amount: 500.into() },
                event_data
            ) =>
            true,
        _ => false,
    });
    assert!(match events.get(1) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<fungible_vault::PayFeeEvent>(event_identifier)
            && is_decoded_equal(
                &fungible_vault::PayFeeEvent {
                    amount: receipt.fee_summary.total_cost()
                },
                event_data
            ) =>
            true,
        _ => false,
    });
    assert!(match events.get(2) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier)
            && is_decoded_equal(
                &fungible_vault::DepositEvent {
                    amount: receipt
                        .expect_commit_success()
                        .fee_destination
                        .to_proposer
                        .checked_add(
                            receipt
                                .expect_commit_success()
                                .fee_destination
                                .to_validator_set
                        )
                        .unwrap()
                },
                event_data
            ) =>
            true,
        _ => false,
    });
    assert!(match events.get(3) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<BurnFungibleResourceEvent>(event_identifier)
            && is_decoded_equal(
                &BurnFungibleResourceEvent {
                    amount: receipt.expect_commit_success().fee_destination.to_burn
                },
                event_data
            ) =>
            true,
        _ => false,
    });
}

//=========
// Scrypto
//=========

#[test]
fn scrypto_cant_emit_unregistered_event() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("events"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ScryptoEvents",
            "emit_unregistered_event",
            manifest_args!(12u64),
        )
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::TypeCheckError(
            TypeCheckError::BlueprintPayloadDoesNotExist(
                _,
                BlueprintPayloadIdentifier::Event(event),
            ),
        )) if event.eq("UnregisteredEvent") => true,
        _ => false,
    });
}

#[test]
fn scrypto_can_emit_registered_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("events"));

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .call_function(
            package_address,
            "ScryptoEvents",
            "emit_registered_event",
            manifest_args!(12u64),
        )
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let events = receipt.expect_commit(true).application_events.clone();
    for event in &events {
        let name = ledger.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
    }
    assert!(match events.get(0) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
            && is_decoded_equal(
                &fungible_vault::LockFeeEvent { amount: 500.into() },
                event_data
            ) =>
            true,
        _ => false,
    });
    assert!(match events.get(1) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Function(blueprint_id), ..),
            ref event_data,
        )) if ledger.is_event_name_equal::<RegisteredEvent>(event_identifier)
            && is_decoded_equal(&RegisteredEvent { number: 12 }, event_data)
            && blueprint_id.package_address == package_address
            && blueprint_id.blueprint_name.eq("ScryptoEvents") =>
            true,
        _ => false,
    });
}

#[test]
fn cant_publish_a_package_with_non_struct_or_enum_event() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let (code, definition) = PackageLoader::get("events_invalid");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidEventSchema,
            )),
        )
    });
}

#[test]
fn local_type_id_with_misleading_name_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let (code, mut definition) = PackageLoader::get("events");
    let blueprint_setup = definition.blueprints.get_mut("ScryptoEvents").unwrap();
    blueprint_setup.schema.events.event_schema.insert(
        "HelloHelloEvent".to_string(),
        blueprint_setup
            .schema
            .events
            .event_schema
            .get("RegisteredEvent")
            .unwrap()
            .clone(),
    );

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::EventNameMismatch { .. },
            )),
        )
    });
}

//=======
// Vault
//=======

#[test]
fn locking_fee_against_a_vault_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let manifest = ManifestBuilder::new().lock_fee(FAUCET, 500).build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn vault_fungible_recall_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let recallable_resource_address = ledger.create_recallable_token(account);
    let vault_id = ledger.get_component_vaults(account, recallable_resource_address)[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .recall(InternalAddress::new_or_panic(vault_id.into()), 1)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::RecallEvent>(event_identifier)
                && is_decoded_equal(&fungible_vault::RecallEvent::new(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier)
                && is_decoded_equal(&fungible_vault::DepositEvent::new(1.into()), event_data) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn vault_non_fungible_recall_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let (recallable_resource_address, non_fungible_local_id) = {
        let id = NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(1));

        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 500)
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                NonFungibleResourceRoles {
                    recall_roles: recall_roles! {
                        recaller => rule!(allow_all);
                        recaller_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                Some([(id.clone(), EmptyStruct {})]),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![]);
        (receipt.expect_commit(true).new_resource_addresses()[0], id)
    };
    let vault_id = ledger.get_component_vaults(account, recallable_resource_address)[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .recall(InternalAddress::new_or_panic(vault_id.into()), 1)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger
                .is_event_name_equal::<non_fungible_vault::RecallEvent>(event_identifier)
                && is_decoded_equal(
                    &non_fungible_vault::RecallEvent::new(indexset!(NonFungibleLocalId::integer(
                        1
                    ))),
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger
                .is_event_name_equal::<non_fungible_vault::DepositEvent>(event_identifier)
                && is_decoded_equal(
                    &non_fungible_vault::DepositEvent::new(indexset!(non_fungible_local_id)),
                    event_data
                ) =>
                true,
            _ => false,
        });
    }
}

//==================
// Resource Manager
//==================

#[test]
fn resource_manager_new_vault_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_fungible_resource(
            OwnerRole::None,
            false,
            18,
            FungibleResourceRoles::default(),
            metadata!(),
            Some(1.into()),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_node_id, ModuleId::Main),
                    ..,
                ),
                ..,
            )) if ledger.is_event_name_equal::<MintFungibleResourceEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_node_id, ModuleId::Main),
                    ..,
                ),
                ..,
            )) if ledger.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier)
                && is_decoded_equal(&fungible_vault::DepositEvent::new(1.into()), event_data) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn resource_manager_mint_and_burn_fungible_resource_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 500)
            .create_fungible_resource(
                OwnerRole::None,
                false,
                18,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                None,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    };

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .mint_fungible(resource_address, 10)
        .burn_from_worktop(10, resource_address)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<MintFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &MintFungibleResourceEvent { amount: 10.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<BurnFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &BurnFungibleResourceEvent { amount: 10.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn resource_manager_mint_and_burn_non_fungible_resource_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 500)
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                NonFungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                None::<BTreeMap<NonFungibleLocalId, EmptyStruct>>,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    };

    let id = NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(1));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .mint_non_fungible(resource_address, [(id.clone(), EmptyStruct {})])
        .burn_from_worktop(1, resource_address)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<MintNonFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &MintNonFungibleResourceEvent {
                        ids: indexset!(id.clone())
                    },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<BurnNonFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &BurnNonFungibleResourceEvent { ids: indexset!(id) },
                    event_data
                ) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn vault_take_non_fungibles_by_amount_emits_correct_event() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(false);
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), dec!("100"))
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles {
                    mint_roles: Some(MintRoles {
                        minter: Some(rule!(allow_all)),
                        minter_updater: None,
                    }),
                    ..Default::default()
                },
                Default::default(),
                None::<BTreeMap<NonFungibleLocalId, EmptyStruct>>,
            )
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(
                    ManifestExpression::EntireWorktop,
                    Option::<ResourceOrNonFungible>::None
                ),
            )
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    };

    let id = NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(1));
    let id2 = NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(2));
    let manifest = ManifestBuilder::new()
        .lock_fee(ledger.faucet_component(), dec!("10"))
        .mint_non_fungible(
            resource_address,
            [(id.clone(), EmptyStruct {}), (id2.clone(), EmptyStruct {})],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .withdraw_from_account(account, resource_address, dec!("2"))
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 10.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<MintNonFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &MintNonFungibleResourceEvent {
                        ids: indexset!(id.clone(), id2.clone())
                    },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                _,
            )) if ledger.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger
                .is_event_name_equal::<non_fungible_vault::DepositEvent>(event_identifier)
                && is_decoded_equal(
                    &non_fungible_vault::DepositEvent::new(indexset!(id.clone(), id2.clone())),
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(4) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<account::DepositEvent>(event_identifier)
                && is_decoded_equal(
                    &account::DepositEvent::NonFungible(
                        resource_address,
                        indexset!(id.clone(), id2.clone())
                    ),
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(5) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger
                .is_event_name_equal::<non_fungible_vault::WithdrawEvent>(event_identifier)
                && is_decoded_equal(
                    &non_fungible_vault::WithdrawEvent::new(indexset!(id.clone(), id2.clone())),
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(7) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger
                .is_event_name_equal::<non_fungible_vault::DepositEvent>(event_identifier)
                && is_decoded_equal(
                    &non_fungible_vault::DepositEvent::new(indexset!(id, id2)),
                    event_data
                ) =>
                true,
            _ => false,
        });
    }
}

//===============
// Consensus Manager
//===============

#[test]
fn consensus_manager_round_update_emits_correct_event() {
    let genesis = BabylonSettings::test_default().with_consensus_manager_config(
        ConsensusManagerConfig::test_default().with_epoch_change_condition(EpochChangeCondition {
            min_round_count: 100, // we do not want the "epoch change" event here
            max_round_count: 100,
            target_duration_millis: 1000,
        }),
    );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();

    // Act
    let receipt = ledger.execute_system_transaction(
        ManifestBuilder::new_system_v1()
            .call_method(
                CONSENSUS_MANAGER,
                CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
                ConsensusManagerNextRoundInput::successful(Round::of(1), 0, 180000i64),
            )
            .build(),
        btreeset![system_execution(SystemExecution::Validator)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<RoundChangeEvent>(event_identifier)
                && is_decoded_equal(
                    &RoundChangeEvent {
                        round: Round::of(1)
                    },
                    event_data
                ) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn consensus_manager_epoch_update_emits_epoch_change_event() {
    let genesis_epoch = Epoch::of(3);
    let initial_epoch = genesis_epoch.next().unwrap();
    let rounds_per_epoch = 5;
    let genesis = BabylonSettings::test_default()
        .with_genesis_epoch(genesis_epoch)
        .with_consensus_manager_config(
            ConsensusManagerConfig::test_default().with_epoch_change_condition(
                EpochChangeCondition {
                    min_round_count: rounds_per_epoch,
                    max_round_count: rounds_per_epoch,
                    target_duration_millis: 1000,
                },
            ),
        );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();

    // Prepare: skip a few rounds, right to the one just before epoch change
    ledger.advance_to_round(Round::of(rounds_per_epoch - 1));

    // Act: perform the most usual successful next round
    let receipt = ledger.execute_system_transaction(
        ManifestBuilder::new_system_v1()
            .call_method(
                CONSENSUS_MANAGER,
                CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
                ConsensusManagerNextRoundInput::successful(
                    Round::of(rounds_per_epoch),
                    0,
                    180000i64,
                ),
            )
            .build(),
        btreeset![system_execution(SystemExecution::Validator)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        let epoch_change_events = events
            .into_iter()
            .filter(|(id, _data)| ledger.is_event_name_equal::<EpochChangeEvent>(id))
            .map(|(_id, data)| scrypto_decode::<EpochChangeEvent>(&data).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(epoch_change_events.len(), 1);
        let event = epoch_change_events.first().unwrap();
        assert_eq!(event.epoch, initial_epoch.next().unwrap());
    }
}

#[test]
fn consensus_manager_epoch_update_emits_xrd_minting_event() {
    // Arrange: some validator, and a degenerate 1-round epoch config, to advance it easily
    let emission_xrd = dec!("13.37");
    let validator_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let genesis = BabylonSettings::single_validator_and_staker(
        validator_key,
        Decimal::one(),
        Decimal::ZERO,
        ComponentAddress::preallocated_account_from_public_key(&validator_key),
        Epoch::of(4),
        ConsensusManagerConfig::test_default()
            .with_epoch_change_condition(EpochChangeCondition {
                min_round_count: 1,
                max_round_count: 1, // deliberate, to go through rounds/epoch without gaps
                target_duration_millis: 0,
            })
            .with_total_emission_xrd_per_epoch(emission_xrd),
    );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();

    // Act
    let receipt = ledger.execute_system_transaction(
        ManifestBuilder::new_system_v1()
            .call_method(
                CONSENSUS_MANAGER,
                CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
                ConsensusManagerNextRoundInput::successful(Round::of(1), 0, 180000i64),
            )
            .build(),
        btreeset![system_execution(SystemExecution::Validator)],
    );

    // Assert
    let result = receipt.expect_commit_success();
    assert_eq!(
        ledger.extract_events_of_type::<MintFungibleResourceEvent>(result),
        vec![
            MintFungibleResourceEvent {
                amount: emission_xrd
            }, // we mint XRD (because of emission)
            MintFungibleResourceEvent {
                amount: emission_xrd
            } // we stake them all immediately because of validator fee = 100% (and thus mint stake units)
        ]
    );
}

//===========
// Validator
//===========

#[test]
fn validator_registration_emits_correct_event() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let genesis = BabylonSettings::test_default().with_genesis_epoch(initial_epoch);
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();
    let (account_pk, _, account) = ledger.new_account(false);

    // Act
    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .register_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<RegisterValidatorEvent>(event_identifier) => true,
            _ => false,
        });
    }
}

#[test]
fn validator_unregistration_emits_correct_event() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let genesis = BabylonSettings::test_default().with_genesis_epoch(initial_epoch);
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();
    let (account_pk, _, account) = ledger.new_account(false);

    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .register_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .unregister_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<UnregisterValidatorEvent>(event_identifier) => true,
            _ => false,
        });
    }
}

#[test]
fn validator_staking_emits_correct_event() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let genesis = BabylonSettings::test_default().with_genesis_epoch(initial_epoch);
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();
    let (account_pk, _, account) = ledger.new_account(false);

    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .register_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .withdraw_from_account(account, XRD, 100)
        .take_all_from_worktop(XRD, "stake")
        .stake_validator_as_owner(validator_address, "stake")
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::WithdrawEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::WithdrawEvent::new(100.into()),
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<account::WithdrawEvent>(event_identifier)
                && is_decoded_equal(
                    &account::WithdrawEvent::Fungible(XRD, 100.into()),
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<MintFungibleResourceEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(4) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier)
                && is_decoded_equal(&fungible_vault::DepositEvent::new(100.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(5) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<StakeEvent>(event_identifier)
                && is_decoded_equal(
                    &StakeEvent {
                        xrd_staked: 100.into()
                    },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(6) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_node_id, ModuleId::Main),
                    ..,
                ),
                ..,
            )) if ledger.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(7) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn validator_unstake_emits_correct_events() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let num_unstake_epochs = 1;
    let validator_pub_key = Secp256k1PrivateKey::from_u64(2u64).unwrap().public_key();
    let account_pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let account_with_su = ComponentAddress::preallocated_account_from_public_key(&account_pub_key);
    let genesis = BabylonSettings::single_validator_and_staker(
        validator_pub_key,
        Decimal::from(10),
        Decimal::ZERO,
        account_with_su,
        initial_epoch,
        ConsensusManagerConfig::test_default().with_num_unstake_epochs(num_unstake_epochs),
    );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();
    let validator_address = ledger.get_active_validator_with_key(&validator_pub_key);
    let validator_substate = ledger.get_validator_info(validator_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .withdraw_from_account(account_with_su, validator_substate.stake_unit_resource, 1)
        .take_all_from_worktop(validator_substate.stake_unit_resource, "stake_units")
        .unstake_validator(validator_address, "stake_units")
        .try_deposit_entire_worktop_or_abort(account_with_su, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );
    receipt.expect_commit_success();
    ledger.set_current_epoch(initial_epoch.after(1 + num_unstake_epochs).unwrap());

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::WithdrawEvent>(event_identifier)
                && is_decoded_equal(&fungible_vault::WithdrawEvent::new(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<account::WithdrawEvent>(event_identifier)
                && is_decoded_equal(
                    &account::WithdrawEvent::Fungible(
                        validator_substate.stake_unit_resource,
                        1.into()
                    ),
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<BurnFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &BurnFungibleResourceEvent { amount: 1.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(4) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<fungible_vault::WithdrawEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(5) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(6) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(node_id, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<MintNonFungibleResourceEvent>(event_identifier)
                && node_id == validator_substate.claim_nft.as_node_id() =>
                true,
            _ => false,
        });
        assert!(match events.get(7) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<UnstakeEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(8) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_node_id, ModuleId::Main),
                    ..,
                ),
                ..,
            )) if ledger.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(9) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger
                .is_event_name_equal::<non_fungible_vault::DepositEvent>(event_identifier) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn validator_claim_xrd_emits_correct_events() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let num_unstake_epochs = 1;
    let validator_pub_key = Secp256k1PrivateKey::from_u64(2u64).unwrap().public_key();
    let account_pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let account_with_su = ComponentAddress::preallocated_account_from_public_key(&account_pub_key);
    let genesis = BabylonSettings::single_validator_and_staker(
        validator_pub_key,
        Decimal::from(10),
        Decimal::ZERO,
        account_with_su,
        initial_epoch,
        ConsensusManagerConfig::test_default().with_num_unstake_epochs(num_unstake_epochs),
    );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();
    let validator_address = ledger.get_active_validator_with_key(&validator_pub_key);
    let validator_substate = ledger.get_validator_info(validator_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .withdraw_from_account(account_with_su, validator_substate.stake_unit_resource, 1)
        .take_all_from_worktop(validator_substate.stake_unit_resource, "stake_units")
        .unstake_validator(validator_address, "stake_units")
        .try_deposit_entire_worktop_or_abort(account_with_su, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );
    receipt.expect_commit_success();
    ledger.set_current_epoch(initial_epoch.after(1 + num_unstake_epochs).unwrap());

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .withdraw_from_account(account_with_su, validator_substate.claim_nft, 1)
        .take_all_from_worktop(validator_substate.claim_nft, "unstake_nft")
        .claim_xrd(validator_address, "unstake_nft")
        .try_deposit_entire_worktop_or_abort(account_with_su, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger
                .is_event_name_equal::<non_fungible_vault::WithdrawEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<account::WithdrawEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<BurnNonFungibleResourceEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(4) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<fungible_vault::WithdrawEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(5) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<ClaimXrdEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(6) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ..,
            )) if ledger.is_event_name_equal::<fungible_vault::DepositEvent>(event_identifier) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn validator_update_stake_delegation_status_emits_correct_event() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_account(false);

    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .register_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .call_method(
            validator_address,
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
            ValidatorUpdateAcceptDelegatedStakeInput {
                accept_delegated_stake: false,
            },
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<UpdateAcceptingStakeDelegationStateEvent>(
                event_identifier
            ) && is_decoded_equal(
                &UpdateAcceptingStakeDelegationStateEvent {
                    accepts_delegation: false
                },
                event_data
            ) =>
                true,
            _ => false,
        });
    }
}

//==========
// Metadata
//==========

#[test]
fn setting_metadata_emits_correct_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let resource_address = create_all_allowed_resource(&mut ledger);

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500)
        .set_metadata(resource_address, "key", MetadataValue::I32(1))
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for event in &events {
            let name = ledger.event_name(&event.0);
            println!("{:?} - {}", event.0, name);
        }
        assert!(match events.get(0) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Main), ..),
                ref event_data,
            )) if ledger.is_event_name_equal::<fungible_vault::LockFeeEvent>(event_identifier)
                && is_decoded_equal(
                    &fungible_vault::LockFeeEvent { amount: 500.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(Emitter::Method(_, ModuleId::Metadata), ..),
                ..,
            )) if ledger.is_event_name_equal::<SetMetadataEvent>(event_identifier) => true,
            _ => false,
        });
    }
}

//=========
// Account
//=========

#[test]
fn create_account_events_can_be_looked_up() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .new_account_advanced(OwnerRole::Fixed(AccessRule::AllowAll), None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for (event_id, _) in events {
            let _name = ledger.event_name(&event_id);
        }
    }
}

//=========
// Helpers
//=========

#[derive(ScryptoSbor, NonFungibleData, ManifestSbor)]
struct EmptyStruct {}

#[derive(ScryptoSbor, PartialEq, Eq, PartialOrd, Ord)]
struct RegisteredEvent {
    number: u64,
}

fn is_decoded_equal<T: ScryptoDecode + PartialEq>(expected: &T, actual: &[u8]) -> bool {
    scrypto_decode::<T>(&actual).unwrap() == *expected
}

fn create_all_allowed_resource(ledger: &mut DefaultLedgerSimulator) -> ResourceAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            OwnerRole::Fixed(AccessRule::AllowAll),
            false,
            18,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => rule!(allow_all);
                    minter_updater => rule!(deny_all);
                },
                burn_roles: burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                },
                recall_roles: recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            metadata!(),
            None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit(true).new_resource_addresses()[0]
}

#[test]
fn mint_burn_events_should_match_total_supply_for_fungible_resource() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_allocated_account();

    // Create
    let resource_address = ledger.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(100)),
        18,
        account,
    );

    // Mint
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_fungible(resource_address, dec!(30))
        .deposit_entire_worktop(account)
        .build();
    ledger
        .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)])
        .expect_commit_success();

    // Burn
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, dec!(10))
        .burn_all_from_worktop(resource_address)
        .build();
    ledger
        .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)])
        .expect_commit_success();

    // Assert
    let mut total_supply = Decimal::ZERO;
    let mut total_mint_amount = Decimal::ZERO;
    let mut total_burn_amount = Decimal::ZERO;
    for component in ledger.find_all_components() {
        let balance = ledger.get_component_balance(component, resource_address);
        total_supply = total_supply.checked_add(balance).unwrap();
        println!("{:?}, {}", component, balance);
    }
    for tx_events in ledger.collected_events() {
        for event in tx_events {
            match &event.0 .0 {
                Emitter::Method(x, _) if x.eq(resource_address.as_node_id()) => {}
                _ => {
                    continue;
                }
            }
            let actual_type_name = ledger.event_name(&event.0);
            match actual_type_name.as_str() {
                "MintFungibleResourceEvent" => {
                    total_mint_amount = total_mint_amount
                        .checked_add(
                            scrypto_decode::<MintFungibleResourceEvent>(&event.1)
                                .unwrap()
                                .amount,
                        )
                        .unwrap();
                }
                "BurnFungibleResourceEvent" => {
                    total_burn_amount = total_burn_amount
                        .checked_add(
                            scrypto_decode::<BurnFungibleResourceEvent>(&event.1)
                                .unwrap()
                                .amount,
                        )
                        .unwrap();
                }
                _ => {}
            }
        }
    }
    println!("Total supply: {}", total_supply);
    println!("Total mint amount: {}", total_mint_amount);
    println!("Total burn amount: {}", total_burn_amount);
    assert_eq!(
        total_supply,
        total_mint_amount.checked_sub(total_burn_amount).unwrap()
    );

    // Query total supply from the resource manager
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(resource_address, "get_total_supply", manifest_args!())
            .build(),
        vec![],
    );
    assert_eq!(
        Some(total_supply),
        receipt.expect_commit_success().output::<Option<Decimal>>(1)
    );
}

#[test]
fn mint_burn_events_should_match_total_supply_for_non_fungible_resource() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_allocated_account();

    // Create
    let resource_address = ledger.create_freely_mintable_and_burnable_non_fungible_resource(
        OwnerRole::None,
        NonFungibleIdType::Integer,
        Some(vec![
            (NonFungibleLocalId::integer(1), EmptyNonFungibleData {}),
            (NonFungibleLocalId::integer(2), EmptyNonFungibleData {}),
            (NonFungibleLocalId::integer(3), EmptyNonFungibleData {}),
        ]),
        account,
    );

    // Mint
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_non_fungible(
            resource_address,
            vec![
                (NonFungibleLocalId::integer(4), EmptyNonFungibleData {}),
                (NonFungibleLocalId::integer(5), EmptyNonFungibleData {}),
            ],
        )
        .deposit_entire_worktop(account)
        .build();
    ledger
        .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)])
        .expect_commit_success();

    // Burn
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(
            account,
            resource_address,
            [NonFungibleLocalId::integer(4)],
        )
        .burn_all_from_worktop(resource_address)
        .build();
    ledger
        .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)])
        .expect_commit_success();

    // Assert
    let mut total_supply = Decimal::ZERO;
    let mut total_mint_non_fungibles = BTreeSet::new();
    let mut total_burn_non_fungibles = BTreeSet::new();
    for component in ledger.find_all_components() {
        let balance = ledger.get_component_balance(component, resource_address);
        total_supply = total_supply.checked_add(balance).unwrap();
        println!("{:?}, {}", component, balance);
    }
    for tx_events in ledger.collected_events() {
        for event in tx_events {
            match &event.0 .0 {
                Emitter::Method(x, _) if x.eq(resource_address.as_node_id()) => {}
                _ => {
                    continue;
                }
            }
            let actual_type_name = ledger.event_name(&event.0);
            match actual_type_name.as_str() {
                "MintNonFungibleResourceEvent" => {
                    total_mint_non_fungibles.extend(
                        scrypto_decode::<MintNonFungibleResourceEvent>(&event.1)
                            .unwrap()
                            .ids,
                    );
                }
                "BurnNonFungibleResourceEvent" => {
                    total_burn_non_fungibles.extend(
                        scrypto_decode::<BurnNonFungibleResourceEvent>(&event.1)
                            .unwrap()
                            .ids,
                    );
                }
                _ => {}
            }
        }
    }
    println!("Total supply: {}", total_supply);
    println!("Total mint: {:?}", total_mint_non_fungibles);
    println!("Total burn: {:?}", total_burn_non_fungibles);
    total_mint_non_fungibles.retain(|x| !total_burn_non_fungibles.contains(x));
    assert_eq!(total_supply, total_mint_non_fungibles.len().into());

    // Query total supply from the resource manager
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(resource_address, "get_total_supply", manifest_args!())
            .build(),
        vec![],
    );
    assert_eq!(
        Some(total_supply),
        receipt.expect_commit_success().output::<Option<Decimal>>(1)
    );
}

#[test]
fn account_withdraw_and_deposit_fungibles_should_emit_correct_event() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, XRD, 1)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    // Assert
    let events = receipt
        .expect_commit_success()
        .application_events
        .as_slice();

    let [
        vault_withdraw_event,
        account_withdraw_event,
        vault_deposit_event,
        account_deposit_event,
        // Note that nobody is paying fee, because of free credit
        _, // receive fee
        _, // burn
    ] = events else {
        panic!("Incorrect number of events: {}", events.len())
    };

    {
        assert_eq!(
            ledger.event_name(&vault_withdraw_event.0),
            fungible_vault::WithdrawEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<fungible_vault::WithdrawEvent>(&vault_withdraw_event.1).unwrap(),
            fungible_vault::WithdrawEvent::new(dec!("1"))
        )
    }
    {
        assert_eq!(
            ledger.event_name(&account_withdraw_event.0),
            account::WithdrawEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::WithdrawEvent>(&account_withdraw_event.1).unwrap(),
            account::WithdrawEvent::Fungible(XRD, dec!("1"))
        )
    }
    {
        assert_eq!(
            ledger.event_name(&vault_deposit_event.0),
            fungible_vault::DepositEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<fungible_vault::DepositEvent>(&vault_deposit_event.1).unwrap(),
            fungible_vault::DepositEvent::new(dec!("1"))
        )
    }
    {
        assert_eq!(
            ledger.event_name(&account_deposit_event.0),
            account::DepositEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::DepositEvent>(&account_deposit_event.1).unwrap(),
            account::DepositEvent::Fungible(XRD, dec!("1"))
        )
    }
}

#[test]
fn account_withdraw_and_deposit_non_fungibles_should_emit_correct_event() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, resource_address, 2)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    // Assert
    let events = receipt
        .expect_commit_success()
        .application_events
        .as_slice();

    let [
        vault_withdraw_event,
        account_withdraw_event,
        vault_deposit_event,
        account_deposit_event,
        // Note that nobody is paying fee, because of free credit
        _, // receive fee
        _, // burn
    ] = events else {
        panic!("Incorrect number of events: {}", events.len())
    };

    let expected_non_fungibles = indexset!(
        NonFungibleLocalId::integer(3),
        NonFungibleLocalId::integer(2)
    );
    {
        assert_eq!(
            ledger.event_name(&vault_withdraw_event.0),
            non_fungible_vault::WithdrawEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<non_fungible_vault::WithdrawEvent>(&vault_withdraw_event.1).unwrap(),
            non_fungible_vault::WithdrawEvent::new(expected_non_fungibles.clone())
        )
    }
    {
        assert_eq!(
            ledger.event_name(&account_withdraw_event.0),
            account::WithdrawEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::WithdrawEvent>(&account_withdraw_event.1).unwrap(),
            account::WithdrawEvent::NonFungible(resource_address, expected_non_fungibles.clone())
        )
    }
    {
        assert_eq!(
            ledger.event_name(&vault_deposit_event.0),
            non_fungible_vault::DepositEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<non_fungible_vault::DepositEvent>(&vault_deposit_event.1).unwrap(),
            non_fungible_vault::DepositEvent::new(expected_non_fungibles.clone())
        )
    }
    {
        assert_eq!(
            ledger.event_name(&account_deposit_event.0),
            account::DepositEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::DepositEvent>(&account_deposit_event.1).unwrap(),
            account::DepositEvent::NonFungible(resource_address, expected_non_fungibles)
        )
    }
}

#[test]
fn account_configuration_emits_expected_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);
    let authorized_depositor_badge = ResourceOrNonFungible::Resource(resource_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
            AccountSetResourcePreferenceInput {
                resource_address,
                resource_preference: ResourcePreference::Allowed,
            },
        )
        .call_method(
            account,
            ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
            AccountSetResourcePreferenceInput {
                resource_address,
                resource_preference: ResourcePreference::Disallowed,
            },
        )
        .call_method(
            account,
            ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT,
            AccountRemoveResourcePreferenceInput { resource_address },
        )
        .call_method(
            account,
            ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
            AccountSetDefaultDepositRuleInput {
                default: DefaultDepositRule::Accept,
            },
        )
        .call_method(
            account,
            ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
            AccountSetDefaultDepositRuleInput {
                default: DefaultDepositRule::Reject,
            },
        )
        .call_method(
            account,
            ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
            AccountSetDefaultDepositRuleInput {
                default: DefaultDepositRule::AllowExisting,
            },
        )
        .call_method(
            account,
            ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
            AccountAddAuthorizedDepositorInput {
                badge: authorized_depositor_badge.clone(),
            },
        )
        .call_method(
            account,
            ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR_IDENT,
            AccountRemoveAuthorizedDepositorInput {
                badge: authorized_depositor_badge.clone(),
            },
        )
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    // Assert
    let events = receipt
        .expect_commit_success()
        .application_events
        .as_slice();

    for event in events {
        let name = ledger.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
    }

    let [
        set_resource_preference_allowed_event,
        set_resource_preference_disallowed_event,

        remove_resource_preference_event,

        set_default_deposit_rule_accept_event,
        set_default_deposit_rule_reject_event,
        set_default_deposit_rule_allow_existing_event,

        add_authorized_depositor_event,
        remove_authorized_depositor_event,

        // Note that nobody is paying fee, because of free credit
        _, // receive fee
        _, // burn
    ] = events else {
        panic!("Incorrect number of events: {}", events.len())
    };

    {
        assert_eq!(
            set_resource_preference_allowed_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&set_resource_preference_allowed_event.0),
            account::SetResourcePreferenceEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::SetResourcePreferenceEvent>(
                &set_resource_preference_allowed_event.1
            )
            .unwrap(),
            account::SetResourcePreferenceEvent {
                resource_address,
                preference: ResourcePreference::Allowed
            }
        )
    }
    {
        assert_eq!(
            set_resource_preference_disallowed_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&set_resource_preference_disallowed_event.0),
            account::SetResourcePreferenceEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::SetResourcePreferenceEvent>(
                &set_resource_preference_disallowed_event.1
            )
            .unwrap(),
            account::SetResourcePreferenceEvent {
                resource_address,
                preference: ResourcePreference::Disallowed
            }
        )
    }
    {
        assert_eq!(
            remove_resource_preference_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&remove_resource_preference_event.0),
            account::RemoveResourcePreferenceEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::RemoveResourcePreferenceEvent>(
                &remove_resource_preference_event.1
            )
            .unwrap(),
            account::RemoveResourcePreferenceEvent { resource_address }
        )
    }
    {
        assert_eq!(
            set_default_deposit_rule_accept_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&set_default_deposit_rule_accept_event.0),
            account::SetDefaultDepositRuleEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::SetDefaultDepositRuleEvent>(
                &set_default_deposit_rule_accept_event.1
            )
            .unwrap(),
            account::SetDefaultDepositRuleEvent {
                default_deposit_rule: DefaultDepositRule::Accept
            }
        )
    }
    {
        assert_eq!(
            set_default_deposit_rule_reject_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&set_default_deposit_rule_reject_event.0),
            account::SetDefaultDepositRuleEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::SetDefaultDepositRuleEvent>(
                &set_default_deposit_rule_reject_event.1
            )
            .unwrap(),
            account::SetDefaultDepositRuleEvent {
                default_deposit_rule: DefaultDepositRule::Reject
            }
        )
    }
    {
        assert_eq!(
            set_default_deposit_rule_allow_existing_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&set_default_deposit_rule_allow_existing_event.0),
            account::SetDefaultDepositRuleEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::SetDefaultDepositRuleEvent>(
                &set_default_deposit_rule_allow_existing_event.1
            )
            .unwrap(),
            account::SetDefaultDepositRuleEvent {
                default_deposit_rule: DefaultDepositRule::AllowExisting
            }
        )
    }
    {
        assert_eq!(
            add_authorized_depositor_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&add_authorized_depositor_event.0),
            account::AddAuthorizedDepositorEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::AddAuthorizedDepositorEvent>(
                &add_authorized_depositor_event.1
            )
            .unwrap(),
            account::AddAuthorizedDepositorEvent {
                authorized_depositor_badge: authorized_depositor_badge.clone()
            }
        )
    }
    {
        assert_eq!(
            remove_authorized_depositor_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&remove_authorized_depositor_event.0),
            account::RemoveAuthorizedDepositorEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::RemoveAuthorizedDepositorEvent>(
                &remove_authorized_depositor_event.1
            )
            .unwrap(),
            account::RemoveAuthorizedDepositorEvent {
                authorized_depositor_badge: authorized_depositor_badge
            }
        )
    }
}

#[test]
fn account_deposit_batch_emits_expected_events() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    for method_name in [
        ACCOUNT_DEPOSIT_BATCH_IDENT,
        ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
        ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
    ] {
        let manifest_args = match method_name {
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT
            | ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT => manifest_args!(
                ManifestExpression::EntireWorktop,
                Option::<ResourceOrNonFungible>::None
            ),
            _ => manifest_args!(ManifestExpression::EntireWorktop),
        };
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(account, XRD, 1)
            .withdraw_from_account(account, resource_address, 3)
            .call_method(account, method_name, manifest_args)
            .build();
        let receipt = ledger.preview_manifest(
            manifest,
            vec![],
            0,
            PreviewFlags {
                use_free_credit: true,
                assume_all_signature_proofs: true,
                skip_epoch_check: true,
                disable_auth: false,
            },
        );

        // Assert
        let events = receipt
            .expect_commit_success()
            .application_events
            .as_slice();
        let [
            _, /* Withdraw of XRD from vault 1 */
            _, /* Withdraw of XRD from account 1 */
            _, /* Withdraw of NFTs from vault 1 */
            _, /* Withdraw of NFTs from account 1 */
            _, /* Deposit of XRD into vault 2 */
            xrd_deposit_event,
            _, /* Deposit of NFTs into vault 2 */
            nfts_deposit_event,
            ..
        ] = events else {
            panic!("Incorrect number of events: {}", events.len())
        };

        {
            assert_eq!(
                xrd_deposit_event.0 .0,
                Emitter::Method(account.into_node_id(), ModuleId::Main)
            );
            assert_eq!(
                ledger.event_name(&xrd_deposit_event.0),
                account::DepositEvent::EVENT_NAME
            );
            assert_eq!(
                scrypto_decode::<account::DepositEvent>(&xrd_deposit_event.1).unwrap(),
                account::DepositEvent::Fungible(XRD, dec!("1"))
            )
        }
        {
            assert_eq!(
                nfts_deposit_event.0 .0,
                Emitter::Method(account.into_node_id(), ModuleId::Main)
            );
            assert_eq!(
                ledger.event_name(&nfts_deposit_event.0),
                account::DepositEvent::EVENT_NAME
            );
            assert_eq!(
                scrypto_decode::<account::DepositEvent>(&nfts_deposit_event.1).unwrap(),
                account::DepositEvent::NonFungible(
                    resource_address,
                    indexset!(
                        NonFungibleLocalId::integer(1),
                        NonFungibleLocalId::integer(2),
                        NonFungibleLocalId::integer(3)
                    )
                )
            )
        }
    }
}

#[test]
fn account_deposit_batch_methods_emits_expected_events_when_deposit_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
            AccountSetDefaultDepositRuleInput {
                default: DefaultDepositRule::Reject,
            },
        )
        .withdraw_from_account(account, XRD, 1)
        .withdraw_from_account(account, resource_address, 3)
        .try_deposit_entire_worktop_or_refund(account, None)
        .deposit_entire_worktop(account)
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    // Assert
    let events = receipt
        .expect_commit_success()
        .application_events
        .as_slice();
    let [
        _, /* Default deposit rule -> Reject */
        _, /* Withdraw of XRD from vault 1 */
        _, /* Withdraw of XRD from account 1 */
        _, /* Withdraw of NFTs from vault 1 */
        _, /* Withdraw of NFTs from account 1 */
        xrd_rejected_deposit_event,
        nfts_rejected_deposit_event,
        ..
    ] = events else {
        panic!("Incorrect number of events: {}", events.len())
    };

    {
        assert_eq!(
            xrd_rejected_deposit_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&xrd_rejected_deposit_event.0),
            account::RejectedDepositEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::RejectedDepositEvent>(&xrd_rejected_deposit_event.1).unwrap(),
            account::RejectedDepositEvent::Fungible(XRD, dec!("1"))
        )
    }
    {
        assert_eq!(
            nfts_rejected_deposit_event.0 .0,
            Emitter::Method(account.into_node_id(), ModuleId::Main)
        );
        assert_eq!(
            ledger.event_name(&nfts_rejected_deposit_event.0),
            account::RejectedDepositEvent::EVENT_NAME
        );
        assert_eq!(
            scrypto_decode::<account::RejectedDepositEvent>(&nfts_rejected_deposit_event.1)
                .unwrap(),
            account::RejectedDepositEvent::NonFungible(
                resource_address,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3)
                )
            )
        )
    }
}

/// A quick into into event replacements and why they take place. Any event that is emitted by a
/// node module prior to it being attached need to have replacements done to their event type
/// identifiers to reflect that. As an example, if the metadata module A was created, emitted an
/// event, and then was attached to component B, then the event emitted by module A should be
/// changed to say that it was emitted by the metadata module of component B.
#[test]
fn event_replacements_occur_as_expected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("event-replacement"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "EventReplacement",
            "instantiate",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let component_address = *receipt
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .unwrap();
    let events = receipt
        .expect_commit_success()
        .application_events
        .as_slice();
    let [
        _, /* Faucet Lock Fee Event */
        (metadata_event_type_identifier, metadata_event_data), /* Withdraw of XRD from vault 1 */
        _, /* Royalty Module vault creation event */
        _, /* Pay Fee Event */
        _, /* Deposit Fee Event */
        _, /* Burn event */
    ] = events else {
        panic!("Incorrect number of events: {}", events.len())
    };
    assert_eq!(
        metadata_event_type_identifier.to_owned(),
        EventTypeIdentifier(
            Emitter::Method(component_address.into_node_id(), ModuleId::Metadata),
            "SetMetadataEvent".to_owned()
        )
    );
    assert_eq!(
        scrypto_decode::<SetMetadataEvent>(&metadata_event_data).unwrap(),
        SetMetadataEvent {
            key: "Hello".to_owned(),
            value: MetadataValue::String("World".to_owned())
        }
    );
}
