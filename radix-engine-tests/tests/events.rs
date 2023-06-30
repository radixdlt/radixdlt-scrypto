use radix_engine::blueprints::consensus_manager::{
    ClaimXrdEvent, EpochChangeEvent, RegisterValidatorEvent, RoundChangeEvent, StakeEvent,
    UnregisterValidatorEvent, UnstakeEvent, UpdateAcceptingStakeDelegationStateEvent,
};
use radix_engine::blueprints::package::PackageError;
use radix_engine::blueprints::resource::*;
use radix_engine::errors::{
    ApplicationError, PayloadValidationAgainstSchemaError, RuntimeError, SystemError,
};
use radix_engine::system::node_modules::metadata::SetMetadataEvent;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerNextRoundInput, EpochChangeCondition, ValidatorUpdateAcceptDelegatedStakeInput,
    CONSENSUS_MANAGER_NEXT_ROUND_IDENT, VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
};
use radix_engine_interface::{metadata, metadata_init};
use scrypto::prelude::Mutability::LOCKED;
use scrypto::prelude::{AccessRule, FromPublicKey, ResourceAction};
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;
use transaction::signing::secp256k1::Secp256k1PrivateKey;

// TODO: In the future, the ClientAPI should only be able to add events to the event store. It
// should not be able to have full control over it.

// FIXME: Creation of proofs triggers withdraw and deposit events when the amount is still liquid.
// This is not the intended behavior. Should figure out a solution to that so that it doesn't emit
// that and clean up this test to have one event.

//=========
// Scrypto
//=========

#[test]
fn scrypto_cant_emit_unregistered_event() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/events");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "ScryptoEvents",
            "emit_unregistered_event",
            manifest_args!(12u64),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
            PayloadValidationAgainstSchemaError::EventDoesNotExist(event),
        )) if event.eq("UnregisteredEvent") => true,
        _ => false,
    });
}

#[test]
fn scrypto_can_emit_registered_events() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/events");

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 500u32.into())
        .call_function(
            package_address,
            "ScryptoEvents",
            "emit_registered_event",
            manifest_args!(12u64),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let events = receipt.expect_commit(true).application_events.clone();
    assert_eq!(events.len(), 2); // Two events: lock fee and registered event
    assert!(match events.get(0) {
        Some((
            event_identifier @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
            ref event_data,
        )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
            && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
            true,
        _ => false,
    });
    assert!(match events.get(1) {
        Some((
            event_identifier @ EventTypeIdentifier(
                Emitter::Function(node_id, ObjectModuleId::Main, blueprint_name),
                ..,
            ),
            ref event_data,
        )) if test_runner.is_event_name_equal::<RegisteredEvent>(event_identifier)
            && is_decoded_equal(&RegisteredEvent { number: 12 }, event_data)
            && node_id == package_address.as_node_id()
            && blueprint_name == "ScryptoEvents" =>
            true,
        _ => false,
    });
}

#[test]
fn cant_publish_a_package_with_non_struct_or_enum_event() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();

    let (code, definition) = Compile::compile("./tests/blueprints/events_invalid");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
fn local_type_index_with_misleading_name_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();

    let (code, mut definition) = Compile::compile("./tests/blueprints/events");
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().without_trace().build();

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 1); // One event: lock fee
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn vault_fungible_recall_emits_correct_events() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (_, _, account) = test_runner.new_account(false);
    let recallable_resource_address = test_runner.create_recallable_token(account);
    let vault_id = test_runner.get_component_vaults(account, recallable_resource_address)[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .recall(InternalAddress::new_or_panic(vault_id.into()), 1.into())
        .call_method(
            account,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 4); // Four events: vault lock fee, vault fungible withdraw, vault fungible recall, vault fungible deposit
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        // FIXME: Currently recall first emits a withdraw event and then a recall event. Should the
        // redundant withdraw event go away or does it make sense from a user perspective?
        assert!(match events.get(1) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<WithdrawResourceEvent>(event_identifier)
                && is_decoded_equal(&WithdrawResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<RecallResourceEvent>(event_identifier)
                && is_decoded_equal(&RecallResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier)
                && is_decoded_equal(&DepositResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
    }
}

// FIXME: double check if the instruction has been updated
// Currently treats non-fungibles as fungible. Correct this test once recall non-fungibles
// has a dedicated instruction.
#[test]
fn vault_non_fungible_recall_emits_correct_events() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (_, _, account) = test_runner.new_account(false);
    let (recallable_resource_address, non_fungible_local_id) = {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceAction::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Recall, (rule!(allow_all), LOCKED));

        let id = NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(1));

        let manifest = ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                metadata!(),
                access_rules,
                Some([(id.clone(), EmptyStruct {})]),
            )
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        (receipt.expect_commit(true).new_resource_addresses()[0], id)
    };
    let vault_id = test_runner.get_component_vaults(account, recallable_resource_address)[0];

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .recall(InternalAddress::new_or_panic(vault_id.into()), 1.into())
        .call_method(
            account,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 4); // Four events: vault lock fee, vault non-fungible withdraw, vault non-fungible recall, vault non-fungible deposit
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        // FIXME: Currently recall first emits a withdraw event and then a recall event. Should the
        // redundant withdraw event go away or does it make sense from a user perspective?
        assert!(match events.get(1) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<WithdrawResourceEvent>(event_identifier)
                && is_decoded_equal(&WithdrawResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<RecallResourceEvent>(event_identifier)
                && is_decoded_equal(&RecallResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &DepositResourceEvent::Ids([non_fungible_local_id.clone()].into()),
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
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (_, _, account) = test_runner.new_account(false);

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_fungible_resource(
            OwnerRole::None,
            false,
            18,
            metadata!(),
            BTreeMap::<ResourceAction, (AccessRule, AccessRule)>::new(),
            Some(1.into()),
        )
        .call_method(
            account,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 3); // Four events: vault lock fee, vault fungible deposit
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_node_id, ObjectModuleId::Main),
                    ..,
                ),
                ..,
            )) if test_runner.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier)
                && is_decoded_equal(&DepositResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn resource_manager_mint_and_burn_fungible_resource_emits_correct_events() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (_, _, account) = test_runner.new_account(false);
    let resource_address = {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceAction::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Mint, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Burn, (rule!(allow_all), LOCKED));

        let manifest = ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .create_fungible_resource(OwnerRole::None, false, 18, metadata!(), access_rules, None)
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    };

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .mint_fungible(resource_address, 10.into())
        .burn_from_worktop(10.into(), resource_address)
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 3); // Three events: vault lock fee, resource manager mint fungible, resource manager burn fungible
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner
                .is_event_name_equal::<MintFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &MintFungibleResourceEvent { amount: 10.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner
                .is_event_name_equal::<BurnFungibleResourceEvent>(event_identifier)
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
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (_, _, account) = test_runner.new_account(false);
    let resource_address = {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceAction::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Mint, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Burn, (rule!(allow_all), LOCKED));

        let manifest = ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                metadata!(),
                access_rules,
                None::<BTreeMap<NonFungibleLocalId, EmptyStruct>>,
            )
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    };

    let id = NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(1));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .mint_non_fungible(resource_address, [(id.clone(), EmptyStruct {})])
        .burn_from_worktop(1.into(), resource_address)
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 3); // Three events: vault lock fee, resource manager mint non-fungible, resource manager burn non-fungible
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner
                .is_event_name_equal::<MintNonFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &MintNonFungibleResourceEvent {
                        ids: [id.clone()].into()
                    },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner
                .is_event_name_equal::<BurnNonFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &BurnNonFungibleResourceEvent {
                        ids: [id.clone()].into()
                    },
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
    let genesis = CustomGenesis::default(
        Epoch::of(1),
        CustomGenesis::default_consensus_manager_config().with_epoch_change_condition(
            EpochChangeCondition {
                min_round_count: 100, // we do not want the "epoch change" event here
                max_round_count: 100,
                target_duration_millis: 1000,
            },
        ),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let receipt = test_runner.execute_validator_transaction(vec![InstructionV1::CallMethod {
        address: CONSENSUS_MANAGER.into(),
        method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value_and_unwrap!(&ConsensusManagerNextRoundInput::successful(
            Round::of(1),
            0,
            180000i64,
        )),
    }]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 1); // One event: round change event
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<RoundChangeEvent>(event_identifier)
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
    let initial_epoch = genesis_epoch.next();
    let rounds_per_epoch = 5;
    let genesis = CustomGenesis::default(
        genesis_epoch,
        CustomGenesis::default_consensus_manager_config().with_epoch_change_condition(
            EpochChangeCondition {
                min_round_count: rounds_per_epoch,
                max_round_count: rounds_per_epoch,
                target_duration_millis: 1000,
            },
        ),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Prepare: skip a few rounds, right to the one just before epoch change
    test_runner.advance_to_round(Round::of(rounds_per_epoch - 1));

    // Act: perform the most usual successful next round
    let receipt = test_runner.execute_validator_transaction(vec![InstructionV1::CallMethod {
        address: CONSENSUS_MANAGER.into(),
        method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value_and_unwrap!(&ConsensusManagerNextRoundInput::successful(
            Round::of(rounds_per_epoch),
            0,
            180000i64,
        )),
    }]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        let epoch_change_events = events
            .into_iter()
            .filter(|(id, _data)| test_runner.is_event_name_equal::<EpochChangeEvent>(id))
            .map(|(_id, data)| scrypto_decode::<EpochChangeEvent>(&data).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(epoch_change_events.len(), 1);
        let event = epoch_change_events.first().unwrap();
        assert_eq!(event.epoch, initial_epoch.next());
    }
}

#[test]
fn consensus_manager_epoch_update_emits_xrd_minting_event() {
    // Arrange: some validator, and a degenerate 1-round epoch config, to advance it easily
    let emission_xrd = dec!("13.37");
    let validator_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        Decimal::one(),
        ComponentAddress::virtual_account_from_public_key(&validator_key),
        Epoch::of(4),
        CustomGenesis::default_consensus_manager_config()
            .with_epoch_change_condition(EpochChangeCondition {
                min_round_count: 1,
                max_round_count: 1, // deliberate, to go through rounds/epoch without gaps
                target_duration_millis: 0,
            })
            .with_total_emission_xrd_per_epoch(emission_xrd),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let receipt = test_runner.execute_validator_transaction(vec![InstructionV1::CallMethod {
        address: CONSENSUS_MANAGER.into(),
        method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value_and_unwrap!(&ConsensusManagerNextRoundInput::successful(
            Round::of(1),
            0,
            180000i64,
        )),
    }]);

    // Assert
    let result = receipt.expect_commit_success();
    assert_eq!(
        test_runner.extract_events_of_type::<MintFungibleResourceEvent>(result),
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
    let genesis = CustomGenesis::default(
        initial_epoch,
        CustomGenesis::default_consensus_manager_config(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let (account_pk, _, account) = test_runner.new_account(false);

    // Act
    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_BADGE)
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 4); // Two events: vault lock fee and register validator
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<RegisterValidatorEvent>(event_identifier) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn validator_unregistration_emits_correct_event() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let genesis = CustomGenesis::default(
        initial_epoch,
        CustomGenesis::default_consensus_manager_config(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let (account_pk, _, account) = test_runner.new_account(false);

    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_BADGE)
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_BADGE)
        .unregister_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 4); // Two events: vault lock fee and register validator
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<UnregisterValidatorEvent>(event_identifier) =>
                true,
            _ => false,
        });
    }
}

#[test]
fn validator_staking_emits_correct_event() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let genesis = CustomGenesis::default(
        initial_epoch,
        CustomGenesis::default_consensus_manager_config(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let (account_pk, _, account) = test_runner.new_account(false);

    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_BADGE)
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_BADGE)
        .withdraw_from_account(account, RADIX_TOKEN, 100.into())
        .take_all_from_worktop(RADIX_TOKEN, |builder, bucket| {
            builder.stake_validator_as_owner(validator_address, bucket)
        })
        .call_method(
            account,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        assert_eq!(events.len(), 9); // Seven events: vault lock fee, vault withdraw fungible, resource manager mint (lp tokens), vault deposit event, validator stake event, resource manager vault create (for the LP tokens), vault deposit
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<WithdrawResourceEvent>(event_identifier)
                && is_decoded_equal(&WithdrawResourceEvent::Amount(100.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner
                .is_event_name_equal::<MintFungibleResourceEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(4) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier)
                && is_decoded_equal(&DepositResourceEvent::Amount(100.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(5) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<StakeEvent>(event_identifier)
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
                    Emitter::Method(_node_id, ObjectModuleId::Main),
                    ..,
                ),
                ..,
            )) if test_runner.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(7) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier) => true,
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
    let account_with_su = ComponentAddress::virtual_account_from_public_key(&account_pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        Decimal::from(10),
        account_with_su,
        initial_epoch,
        CustomGenesis::default_consensus_manager_config()
            .with_num_unstake_epochs(num_unstake_epochs),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(
            account_with_su,
            validator_substate.stake_unit_resource,
            1.into(),
        )
        .take_all_from_worktop(validator_substate.stake_unit_resource, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .call_method(
            account_with_su,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );
    receipt.expect_commit_success();
    test_runner.set_current_epoch(initial_epoch.after(1 + num_unstake_epochs));

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        /*
        Nine Events:
        1. Lock Fee event
        2. Vault withdraw event (LP Tokens)
        3. Resource Manager burn event (LP Tokens)
        4. Vault withdraw event (withdraw from stake vault)
        5. Vault deposit event (deposit into stake vault)
        6. Resource Manager Mint (minting unstake redeem tokens)
        7. Validator Unstake event
        8. Resource Manager Vault creation event (unstake redeem tokens)
        9. Vault Deposit Event (unstake redeem tokens)
         */
        assert_eq!(events.len(), 9);
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<WithdrawResourceEvent>(event_identifier)
                && is_decoded_equal(&WithdrawResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner
                .is_event_name_equal::<BurnFungibleResourceEvent>(event_identifier)
                && is_decoded_equal(
                    &BurnFungibleResourceEvent { amount: 1.into() },
                    event_data
                ) =>
                true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<WithdrawResourceEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(4) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(5) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(node_id, ObjectModuleId::Main),
                    ..,
                ),
                ..,
            )) if test_runner
                .is_event_name_equal::<MintNonFungibleResourceEvent>(event_identifier)
                && node_id == validator_substate.unstake_nft.as_node_id() =>
                true,
            _ => false,
        });
        assert!(match events.get(6) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<UnstakeEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(7) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_node_id, ObjectModuleId::Main),
                    ..,
                ),
                ..,
            )) if test_runner.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(8) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier) => true,
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
    let account_with_su = ComponentAddress::virtual_account_from_public_key(&account_pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        Decimal::from(10),
        account_with_su,
        initial_epoch,
        CustomGenesis::default_consensus_manager_config()
            .with_num_unstake_epochs(num_unstake_epochs),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(
            account_with_su,
            validator_substate.stake_unit_resource,
            1.into(),
        )
        .take_all_from_worktop(validator_substate.stake_unit_resource, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .call_method(
            account_with_su,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );
    receipt.expect_commit_success();
    test_runner.set_current_epoch(initial_epoch.after(1 + num_unstake_epochs));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(account_with_su, validator_substate.unstake_nft, 1.into())
        .take_all_from_worktop(validator_substate.unstake_nft, |builder, bucket| {
            builder.claim_xrd(validator_address, bucket)
        })
        .call_method(
            account_with_su,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        /*
        Seven Events:
        1. Vault lock fee event
        2. Vault withdraw event (unstake nft)
        3. Resource Manager burn event (unstake nft)
        4. Vault withdraw event (unstaked xrd)
        5. Claim XRD
        5. Resource Manager vault creation event (XRD)
        6. Vault deposit event (XRD)
         */
        assert_eq!(events.len(), 7);
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<WithdrawResourceEvent>(event_identifier)
                && is_decoded_equal(&WithdrawResourceEvent::Amount(1.into()), event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner
                .is_event_name_equal::<BurnNonFungibleResourceEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(3) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<WithdrawResourceEvent>(event_identifier) =>
                true,
            _ => false,
        });
        assert!(match events.get(4) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<ClaimXrdEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(5) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_node_id, ObjectModuleId::Main),
                    ..,
                ),
                ..,
            )) if test_runner.is_event_name_equal::<VaultCreationEvent>(event_identifier) => true,
            _ => false,
        });
        assert!(match events.get(6) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ..,
            )) if test_runner.is_event_name_equal::<DepositResourceEvent>(event_identifier) => true,
            _ => false,
        });
    }
}

#[test]
fn validator_update_stake_delegation_status_emits_correct_event() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let genesis = CustomGenesis::default(
        initial_epoch,
        CustomGenesis::default_consensus_manager_config(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let (pub_key, _, account) = test_runner.new_account(false);

    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_BADGE)
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
            to_manifest_value_and_unwrap!(&ValidatorUpdateAcceptDelegatedStakeInput {
                accept_delegated_stake: false,
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        /*
        5 Events:
        1. Vault lock fee event
        2. Withdraw event
        3. Validator update delegation state
        4. Deposit event
         */
        assert_eq!(events.len(), 4);
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(2) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<UpdateAcceptingStakeDelegationStateEvent>(
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
    let mut test_runner = TestRunner::builder().without_trace().build();
    let resource_address = create_all_allowed_resource(&mut test_runner);

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .set_metadata(resource_address, "key", MetadataValue::I32(1))
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        /*
        Two events:
        1. Vault lock fee
        2. Metadata set entry
         */
        assert_eq!(events.len(), 2);
        assert!(match events.get(0) {
            Some((
                event_identifier
                @ EventTypeIdentifier(Emitter::Method(_, ObjectModuleId::Main), ..),
                ref event_data,
            )) if test_runner.is_event_name_equal::<LockFeeEvent>(event_identifier)
                && is_decoded_equal(&LockFeeEvent { amount: 500.into() }, event_data) =>
                true,
            _ => false,
        });
        assert!(match events.get(1) {
            Some((
                event_identifier @ EventTypeIdentifier(
                    Emitter::Method(_, ObjectModuleId::Metadata),
                    ..,
                ),
                ..,
            )) if test_runner.is_event_name_equal::<SetMetadataEvent>(event_identifier) => true,
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
    let mut test_runner = TestRunner::builder().without_trace().build();

    // Act
    let manifest = ManifestBuilder::new()
        .new_account_advanced(OwnerRole::Fixed(AccessRule::AllowAll))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    {
        let events = receipt.expect_commit(true).clone().application_events;
        for (event_id, _) in events {
            let _name = test_runner.event_name(&event_id);
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

fn create_all_allowed_resource(test_runner: &mut TestRunner) -> ResourceAddress {
    let access_rules = [
        ResourceAction::Burn,
        ResourceAction::Deposit,
        ResourceAction::Withdraw,
        ResourceAction::Mint,
        ResourceAction::Burn,
        ResourceAction::UpdateNonFungibleData,
    ]
    .into_iter()
    .map(|method| (method, (AccessRule::AllowAll, AccessRule::AllowAll)))
    .collect();

    let manifest = ManifestBuilder::new()
        .create_fungible_resource(
            OwnerRole::Fixed(AccessRule::AllowAll),
            false,
            18,
            metadata!(),
            access_rules,
            None,
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
    *receipt
        .expect_commit(true)
        .new_resource_addresses()
        .get(0)
        .unwrap()
}
