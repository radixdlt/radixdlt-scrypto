use radix_common::prelude::*;
use radix_engine::blueprints::native_schema::*;
use radix_engine::transaction::TransactionResult;
use radix_engine::updates::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_queries::typed_native_events::TypedNativeEvent;
use radix_transaction_scenarios::executor::*;
use sbor::rust::ops::Deref;
use scrypto_test::prelude::*;

#[test]
fn test_bootstrap_and_protocol_update_receipts_have_substate_changes_which_can_be_typed() {
    let mut substate_db = InMemorySubstateDatabase::standard();

    struct Hooks;
    impl ProtocolUpdateExecutionHooks for Hooks {
        fn on_transaction_executed(&mut self, event: OnProtocolTransactionExecuted) {
            let OnProtocolTransactionExecuted { receipt, .. } = event;
            assert_receipt_substate_changes_can_be_typed(receipt.expect_commit_success())
        }
    }
    let mut hooks = Hooks;
    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings::test_complex())
        .from_bootstrap_to_latest()
        .commit_each_protocol_update_advanced(
            &mut substate_db,
            &mut hooks,
            &mut VmModules::default(),
        );
}

#[test]
fn test_bootstrap_and_protocol_update_receipts_have_events_that_can_be_typed() {
    let mut substate_db = InMemorySubstateDatabase::standard();

    struct Hooks;
    impl ProtocolUpdateExecutionHooks for Hooks {
        fn on_transaction_executed(&mut self, event: OnProtocolTransactionExecuted) {
            let OnProtocolTransactionExecuted { receipt, .. } = event;
            assert_receipt_events_can_be_typed(receipt.expect_commit_success())
        }
    }
    let mut hooks = Hooks;
    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings::test_complex())
        .from_bootstrap_to_latest()
        .commit_each_protocol_update_advanced(
            &mut substate_db,
            &mut hooks,
            &mut VmModules::default(),
        );
}

#[test]
fn test_all_scenario_commit_receipts_should_have_substate_changes_which_can_be_typed() {
    struct Hooks;
    impl<S: SubstateDatabase> ScenarioExecutionHooks<S> for Hooks {
        fn on_transaction_executed(&mut self, event: OnScenarioTransactionExecuted<S>) {
            let OnScenarioTransactionExecuted { receipt, .. } = event;
            if let TransactionResult::Commit(ref commit_result) = receipt.result {
                assert_receipt_substate_changes_can_be_typed(commit_result);
            };
        }
    }

    TransactionScenarioExecutor::new(
        InMemorySubstateDatabase::standard(),
        NetworkDefinition::simulator(),
    )
    .execute_every_protocol_update_and_scenario(&mut Hooks)
    .expect("Must succeed!");
}

#[test]
fn test_all_scenario_commit_receipts_should_have_events_that_can_be_typed() {
    struct Hooks;
    impl<S: SubstateDatabase> ScenarioExecutionHooks<S> for Hooks {
        fn on_transaction_executed(&mut self, event: OnScenarioTransactionExecuted<S>) {
            let OnScenarioTransactionExecuted { receipt, .. } = event;
            if let TransactionResult::Commit(ref commit_result) = receipt.result {
                assert_receipt_events_can_be_typed(commit_result);
            };
        }
    }
    TransactionScenarioExecutor::new(
        InMemorySubstateDatabase::standard(),
        NetworkDefinition::simulator(),
    )
    .execute_every_protocol_update_and_scenario(&mut Hooks)
    .expect("Must succeed!");
}

/// We need to ensure that all of the events registered to native events are included in the typed
/// native event model. This test checks that the events in `typed_native_events.rs` module all
/// exist in the blueprint schema.
#[test]
fn typed_native_event_type_contains_all_native_events() {
    // Arrange
    let package_name_definition_mapping = hashmap! {
        "ConsensusManager" => CONSENSUS_MANAGER_PACKAGE_DEFINITION.deref(),
        "Account" => ACCOUNT_PACKAGE_DEFINITION.deref(),
        "Identity" => IDENTITY_PACKAGE_DEFINITION.deref(),
        "AccessController" => ACCESS_CONTROLLER_PACKAGE_DEFINITION_V2_0.deref(),
        "Pool" => POOL_PACKAGE_DEFINITION_V1_0.deref(),
        "TransactionTracker" => TRANSACTION_TRACKER_PACKAGE_DEFINITION.deref(),
        "Resource" => RESOURCE_PACKAGE_DEFINITION.deref(),
        "Package" => PACKAGE_PACKAGE_DEFINITION.deref(),
        "TransactionProcessor" => TRANSACTION_PROCESSOR_PACKAGE_DEFINITION.deref(),
        "Locker" => LOCKER_PACKAGE_DEFINITION.deref(),
        "Metadata" => METADATA_PACKAGE_DEFINITION.deref(),
        "Royalty" => ROYALTY_PACKAGE_DEFINITION.deref(),
        "RoleAssignment" => ROLE_ASSIGNMENT_PACKAGE_DEFINITION.deref(),
    };

    // Act
    let registered_events = TypedNativeEvent::registered_events();

    // Assert
    for (package_name, package_blueprints) in registered_events.into_iter() {
        let package_definition = package_name_definition_mapping
            .get(package_name.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "No package definition found for a package with the name: \"{package_name}\""
                )
            });
        for (blueprint_name, blueprint_events) in package_blueprints.into_iter() {
            let blueprint_definition = package_definition.blueprints.get(&blueprint_name).unwrap_or_else(|| panic!("Package named \"{package_name}\" has no blueprint named \"{blueprint_name}\""));
            let actual_blueprint_events = blueprint_definition
                .schema
                .events
                .event_schema
                .keys()
                .cloned()
                .collect::<HashSet<_>>();

            assert_eq!(
                blueprint_events,
                actual_blueprint_events,
                "There is a difference between the actual blueprint events and the ones registered in the typed model. Package name: \"{package_name}\", Blueprint name: \"{blueprint_name}\""
            )
        }
    }
}

#[test]
fn access_controller_state_v1_can_be_decoded_as_v2() {
    // Arrange
    use radix_engine::blueprints::access_controller::v1;
    use radix_engine::blueprints::access_controller::v2;

    let value = v1::AccessControllerStateFieldSubstate::V1(FieldSubstateV1 {
        payload: v1::AccessControllerStateFieldPayload::of(
            v1::VersionedAccessControllerState::new(v1::AccessControllerStateVersions::V1(
                v1::AccessControllerV1Substate {
                    controlled_asset: Vault(Own(NodeId(
                        [EntityType::InternalFungibleVault as u8; 30],
                    ))),
                    recovery_badge: ACCOUNT_OWNER_BADGE,
                    state: Default::default(),
                    timed_recovery_delay_in_minutes: Default::default(),
                },
            )),
        ),
        lock_status: LockStatus::Locked,
    });

    // Act
    let decoded = scrypto_decode::<v2::AccessControllerV2StateFieldSubstate>(
        &scrypto_encode(&value).unwrap(),
    );

    // Assert
    let _ = decoded.expect("Must succeed!");
}
