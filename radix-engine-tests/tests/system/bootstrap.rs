use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::object_modules::metadata::*;
use radix_engine::system::bootstrap::*;
use radix_engine::system::checkers::SystemDatabaseChecker;
use radix_engine::system::checkers::{
    ResourceDatabaseChecker, ResourceEventChecker, ResourceReconciler, SystemEventChecker,
};
use radix_engine::system::system_db_reader::{ObjectCollectionKey, SystemDatabaseReader};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::{BalanceChange, CommitResult, SystemStructure};
use radix_engine::updates::{BabylonSettings, ProtocolBuilder};
use radix_engine_interface::object_modules::metadata::{MetadataValue, UncheckedUrl};
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_queries::typed_substate_layout::*;
use radix_transactions::prelude::*;
use scrypto_test::prelude::*;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let vm_modules = VmModules::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let genesis_epoch = Epoch::of(1);
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];

    let mut hooks = GenesisReceiptExtractionHooks::new();
    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings {
            genesis_data_chunks,
            genesis_epoch,
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        })
        .only_babylon()
        .commit_each_protocol_update_advanced(&mut substate_db, &mut hooks, &vm_modules);

    let GenesisReceipts {
        system_flash_receipt,
        system_bootstrap_receipt,
        data_ingestion_receipts,
        wrap_up_receipt,
    } = hooks.into_genesis_receipts();

    assert!(system_flash_receipt
        .expect_commit_success()
        .new_package_addresses()
        .contains(&PACKAGE_PACKAGE));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&GENESIS_HELPER));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_package_addresses()
        .contains(&TRANSACTION_TRACKER_PACKAGE));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&TRANSACTION_TRACKER));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&FAUCET));

    let wrap_up_epoch_change = wrap_up_receipt
        .expect_commit_success()
        .next_epoch()
        .expect("There should be a new epoch.");

    assert_eq!(wrap_up_epoch_change.epoch, genesis_epoch.next().unwrap());

    let mut checker = SystemDatabaseChecker::<ResourceDatabaseChecker>::default();
    let db_results = checker
        .check_db(&substate_db)
        .expect("Database should be consistent");
    println!("{:#?}", db_results);

    let mut event_checker = SystemEventChecker::<ResourceEventChecker>::new();
    let mut events = Vec::new();
    events.push(
        system_bootstrap_receipt
            .expect_commit_success()
            .application_events
            .clone(),
    );
    events.extend(
        data_ingestion_receipts
            .into_iter()
            .map(|r| r.expect_commit_success().application_events.clone()),
    );
    events.push(
        wrap_up_receipt
            .expect_commit_success()
            .application_events
            .clone(),
    );
    let event_results = event_checker
        .check_all_events(&substate_db, &events)
        .expect("Events should be consistent");
    println!("{:#?}", event_results);

    ResourceReconciler::reconcile(&db_results.1, &event_results)
        .expect("Resource reconciliation failed.");
}

#[test]
fn test_bootstrap_receipts_should_have_complete_system_structure() {
    let vm_modules = VmModules::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let genesis_epoch = Epoch::of(1);
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];
    let mut hooks = GenesisReceiptExtractionHooks::new();
    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings {
            genesis_data_chunks,
            genesis_epoch,
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        })
        .only_babylon()
        .commit_each_protocol_update_advanced(&mut substate_db, &mut hooks, &vm_modules);

    let GenesisReceipts {
        system_flash_receipt: _,
        system_bootstrap_receipt,
        data_ingestion_receipts,
        wrap_up_receipt,
    } = hooks.into_genesis_receipts();

    assert_complete_system_structure(system_bootstrap_receipt.expect_commit_success());
    for data_ingestion_receipt in data_ingestion_receipts {
        assert_complete_system_structure(data_ingestion_receipt.expect_commit_success());
    }
    assert_complete_system_structure(wrap_up_receipt.expect_commit_success());
}

// TODO(after RCnet-V3): this assertion could be re-used for other tests of non-standard receipts.
fn assert_complete_system_structure(result: &CommitResult) {
    let SystemStructure {
        substate_system_structures,
        event_system_structures,
    } = &result.system_structure;

    let substate_updates = result
        .state_updates
        .clone()
        .into_flattened_substate_updates();
    for (node_id, partition_num, substate_key) in substate_updates.keys() {
        let structure = substate_system_structures
            .get(node_id)
            .and_then(|partition_structures| partition_structures.get(partition_num))
            .and_then(|substate_structures| substate_structures.get(substate_key));
        assert!(
            structure.is_some(),
            "missing system structure for {:?}:{:?}:{:?}",
            node_id,
            partition_num,
            substate_key
        );
    }

    for (event_type_id, _data) in &result.application_events {
        let structure = event_system_structures.get(event_type_id);
        assert!(
            structure.is_some(),
            "missing system structure for {:?}",
            event_type_id
        );
    }
}

fn test_genesis_resource_with_initial_allocation(owned_resource: bool) {
    let vm_modules = VmModules::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let token_holder = ComponentAddress::preallocated_account_from_public_key(
        &PublicKey::Secp256k1(Secp256k1PrivateKey::from_u64(1).unwrap().public_key()),
    );
    let resource_address = ResourceAddress::new_or_panic(
        NodeId::new(
            EntityType::GlobalFungibleResourceManager as u8,
            &hash(vec![1, 2, 3]).lower_bytes(),
        )
        .0,
    );
    let resource_owner = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(2).unwrap().public_key(),
    );
    let allocation_amount = dec!("105");
    let genesis_resource = GenesisResource {
        reserved_resource_address: resource_address,
        metadata: vec![(
            "symbol".to_string(),
            MetadataValue::String("TST".to_string()),
        )],
        owner: if owned_resource {
            Some(resource_owner)
        } else {
            None
        },
    };
    let resource_allocation = GenesisResourceAllocation {
        account_index: 0,
        amount: allocation_amount,
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Resources(vec![genesis_resource]),
        GenesisDataChunk::ResourceBalances {
            accounts: vec![token_holder.clone()],
            allocations: vec![(resource_address.clone(), vec![resource_allocation])],
        },
    ];

    let mut hooks = GenesisReceiptExtractionHooks::new();
    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings {
            genesis_data_chunks,
            genesis_epoch: Epoch::of(1),
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        })
        .only_babylon()
        .commit_each_protocol_update_advanced(&mut substate_db, &mut hooks, &vm_modules);

    let GenesisReceipts {
        mut data_ingestion_receipts,
        ..
    } = hooks.into_genesis_receipts();

    let total_supply = substate_db
        .get_substate::<FungibleResourceManagerTotalSupplyFieldSubstate>(
            resource_address,
            MAIN_BASE_PARTITION,
            FungibleResourceManagerField::TotalSupply,
        )
        .unwrap()
        .into_payload()
        .fully_update_and_into_latest_version();
    assert_eq!(total_supply, allocation_amount);

    let reader = SystemDatabaseReader::new(&substate_db);
    let entry = reader
        .read_object_collection_entry::<_, MetadataEntryEntryPayload>(
            resource_address.as_node_id(),
            ModuleId::Metadata,
            ObjectCollectionKey::KeyValue(
                MetadataCollection::EntryKeyValue.collection_index(),
                &"symbol".to_string(),
            ),
        )
        .unwrap()
        .map(|v| v.fully_update_and_into_latest_version());

    if let Some(MetadataValue::String(symbol)) = entry {
        assert_eq!(symbol, "TST");
    } else {
        panic!("Resource symbol was not a string");
    }

    let allocation_receipt = data_ingestion_receipts.pop().unwrap();
    let resource_creation_receipt = data_ingestion_receipts.pop().unwrap();

    println!("{:?}", resource_creation_receipt);
    let resource_creation_commit = resource_creation_receipt.expect_commit_success();

    if owned_resource {
        let created_owner_badge = resource_creation_commit.new_resource_addresses()[1];
        let owner_badge_vault = resource_creation_commit.new_vault_addresses()[0];

        // check if the metadata exists and is locked
        let reader = SystemDatabaseReader::new(&substate_db);
        let substate = reader
            .fetch_substate::<KeyValueEntrySubstate<VersionedMetadataEntry>>(
                created_owner_badge.as_node_id(),
                METADATA_BASE_PARTITION,
                &SubstateKey::Map(scrypto_encode("tags").unwrap()),
            )
            .unwrap();
        assert!(substate.is_locked());
        assert_eq!(
            substate.into_value().map(|v| v.into_unique_version()),
            Some(MetadataValue::StringArray(vec!["badge".to_owned()]))
        );

        assert_eq!(
            resource_creation_commit
                .state_update_summary
                .vault_balance_changes
                .get(owner_badge_vault.as_node_id())
                .unwrap(),
            &(created_owner_badge, BalanceChange::Fungible(1.into()))
        );
    }

    let created_resource = resource_creation_commit.new_resource_addresses()[0]; // The resource address is preallocated, thus [0]
    let allocation_commit = allocation_receipt.expect_commit_success();
    let created_vault = allocation_commit.new_vault_addresses()[0];

    assert_eq!(
        allocation_commit
            .state_update_summary
            .vault_balance_changes
            .get(created_vault.as_node_id())
            .unwrap(),
        &(created_resource, BalanceChange::Fungible(allocation_amount))
    );
}

#[test]
// It would be more elegant to validate some return values instead of expecting a panic.
// But since it is a bootstrap stage we believe it is good enough.
#[should_panic(expected = "Failure(ApplicationError(ConsensusManagerError(ExceededValidatorCount")]
fn test_bootstrap_with_exceeded_validator_count() {
    let mut substate_db = InMemorySubstateDatabase::standard();

    let mut initial_config = ConsensusManagerConfig::test_default();

    // exceeding max validator count - expecting a panic now
    initial_config.max_validators = ValidatorIndex::MAX as u32 + 1;

    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings {
            genesis_data_chunks: vec![],
            genesis_epoch: Epoch::of(1),
            consensus_manager_config: initial_config,
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        })
        .only_babylon()
        .commit_each_protocol_update(&mut substate_db);
}

#[test]
fn test_genesis_resource_with_initial_owned_allocation() {
    test_genesis_resource_with_initial_allocation(true);
}

#[test]
fn test_genesis_resource_with_initial_unowned_allocation() {
    test_genesis_resource_with_initial_allocation(false);
}

#[test]
fn test_genesis_stake_allocation() {
    let vm_modules = VmModules::default();
    let mut substate_db = InMemorySubstateDatabase::standard();

    // There are two genesis validators
    // - one with two stakers (0 and 1)
    // - one with one staker (just 1)
    let validator_0_key = Secp256k1PrivateKey::from_u64(10).unwrap().public_key();
    let validator_1_key = Secp256k1PrivateKey::from_u64(11).unwrap().public_key();
    let staker_0 = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );
    let staker_1 = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(5).unwrap().public_key(),
    );
    let validator_0_allocations = vec![
        GenesisStakeAllocation {
            account_index: 0,
            xrd_amount: dec!("10"),
        },
        GenesisStakeAllocation {
            account_index: 1,
            xrd_amount: dec!("50000"),
        },
    ];
    let validator_1_allocations = vec![GenesisStakeAllocation {
        account_index: 1,
        xrd_amount: dec!(1),
    }];
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![
            validator_0_key.clone().into(),
            validator_1_key.clone().into(),
        ]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_0, staker_1],
            allocations: vec![
                (validator_0_key, validator_0_allocations),
                (validator_1_key, validator_1_allocations),
            ],
        },
    ];

    let mut hooks = GenesisReceiptExtractionHooks::new();
    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings {
            genesis_data_chunks,
            genesis_epoch: Epoch::of(1),
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        })
        .only_babylon()
        .commit_each_protocol_update_advanced(&mut substate_db, &mut hooks, &vm_modules);

    let GenesisReceipts {
        mut data_ingestion_receipts,
        ..
    } = hooks.into_genesis_receipts();

    let allocate_stakes_receipt = data_ingestion_receipts.pop().unwrap();

    let commit = allocate_stakes_receipt.expect_commit_success();
    let descendant_vaults = SubtreeVaults::new(&substate_db);

    // Staker 1 should have two liquidity balance entries
    {
        let address: GlobalAddress = staker_1.into();
        let balances = descendant_vaults
            .sum_balance_changes(address.as_node_id(), commit.vault_balance_changes());
        assert_eq!(balances.len(), 2);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!(1))));
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("50000"))));
    }

    let create_validators_receipt = data_ingestion_receipts.pop().unwrap();
    {
        let new_validators: Vec<ComponentAddress> = create_validators_receipt
            .expect_commit_success()
            .state_update_summary
            .new_components
            .iter()
            .filter(|c| c.as_node_id().entity_type() == Some(EntityType::GlobalValidator))
            .cloned()
            .collect();

        let reader = SystemDatabaseReader::new(&substate_db);

        for (index, validator_key) in vec![validator_0_key, validator_1_key]
            .into_iter()
            .enumerate()
        {
            let validator_url_entry = reader
                .read_object_collection_entry::<_, MetadataEntryEntryPayload>(
                    &new_validators[index].as_node_id(),
                    ModuleId::Metadata,
                    ObjectCollectionKey::KeyValue(
                        MetadataCollection::EntryKeyValue.collection_index(),
                        &"url".to_string(),
                    ),
                )
                .unwrap()
                .map(|v| v.fully_update_and_into_latest_version());
            if let Some(MetadataValue::Url(url)) = validator_url_entry {
                assert_eq!(
                    url,
                    UncheckedUrl::of(format!("http://test.local?validator={:?}", validator_key))
                );
            } else {
                panic!("Validator url was not a Url");
            }
        }
    }
}

#[test]
fn test_genesis_time() {
    let mut substate_db = InMemorySubstateDatabase::standard();

    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings {
            genesis_data_chunks: vec![],
            genesis_epoch: Epoch::of(1),
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 123 * 60 * 1000 + 22, // 123 full minutes + 22 ms (which should be rounded down)
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        })
        .only_babylon()
        .commit_each_protocol_update(&mut substate_db);

    let reader = SystemDatabaseReader::new(&mut substate_db);
    let timestamp = reader
        .read_typed_object_field::<ConsensusManagerProposerMinuteTimestampFieldPayload>(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::ProposerMinuteTimestamp.field_index(),
        )
        .unwrap()
        .fully_update_and_into_latest_version();

    assert_eq!(timestamp.epoch_minute, 123);
}

#[test]
fn should_not_be_able_to_create_genesis_helper() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            GENESIS_HELPER_PACKAGE,
            GENESIS_HELPER_BLUEPRINT,
            "new",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_call_genesis_helper() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(GENESIS_HELPER, "wrap_up", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn mint_burn_events_should_match_resource_supply_post_genesis_and_notarized_tx() {
    // Arrange
    // Data migrated from Olympia
    let validator_0_key = Secp256k1PrivateKey::from_u64(10).unwrap().public_key();
    let validator_1_key = Secp256k1PrivateKey::from_u64(11).unwrap().public_key();
    let staker_0 = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );
    let staker_1 = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(5).unwrap().public_key(),
    );
    let validator_0_allocations = vec![
        GenesisStakeAllocation {
            account_index: 0,
            xrd_amount: dec!("10"),
        },
        GenesisStakeAllocation {
            account_index: 1,
            xrd_amount: dec!("100"),
        },
    ];
    let validator_1_allocations = vec![GenesisStakeAllocation {
        account_index: 1,
        xrd_amount: dec!(2),
    }];
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![
            validator_0_key.clone().into(),
            validator_1_key.clone().into(),
        ]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_0, staker_1],
            allocations: vec![
                (validator_0_key, validator_0_allocations),
                (validator_1_key, validator_1_allocations),
            ],
        },
        GenesisDataChunk::XrdBalances(vec![(staker_0, dec!(200)), (staker_1, dec!(300))]),
    ];

    let genesis = BabylonSettings {
        genesis_data_chunks,
        genesis_epoch: Epoch::of(1),
        consensus_manager_config: ConsensusManagerConfig::test_default(),
        initial_time_ms: 0,
        initial_current_leader: Some(0),
        faucet_supply: *DEFAULT_TESTING_FAUCET_SUPPLY,
    };

    // Bootstrap
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.configure_babylon(|_| genesis).only_babylon())
        .build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .drop_auth_zone_proofs()
        .build();
    ledger.execute_manifest(manifest, vec![]);

    // Assert
    println!("Staker 0: {:?}", staker_0);
    println!("Staker 1: {:?}", staker_1);
    let components = ledger.find_all_components();
    let mut total_xrd_supply = Decimal::ZERO;
    for component in components {
        let xrd_balance = ledger.get_component_balance(component, XRD);
        total_xrd_supply = total_xrd_supply.checked_add(xrd_balance).unwrap();
        println!("{:?}, {}", component, xrd_balance);
    }

    let mut total_mint_amount = Decimal::ZERO;
    let mut total_burn_amount = Decimal::ZERO;
    for tx_events in ledger.collected_events() {
        for event in tx_events {
            match &event.0 .0 {
                Emitter::Method(x, _) if x.eq(XRD.as_node_id()) => {}
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
    println!("Total XRD supply: {}", total_xrd_supply);
    println!("Total mint amount: {}", total_mint_amount);
    println!("Total burn amount: {}", total_burn_amount);
    assert_eq!(
        total_xrd_supply,
        total_mint_amount.checked_sub(total_burn_amount).unwrap()
    );
}

#[test]
fn test_bootstrap_should_create_consensus_manager_with_sorted_validator_index() {
    let mut substate_db = InMemorySubstateDatabase::standard();
    let staker_address = ComponentAddress::preallocated_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let validator_key = Secp256k1PublicKey([7; 33]);
    let stake_xrd = dec!("1337");
    let stake_allocation = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: stake_xrd,
    };
    let validator_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake_allocation])],
        },
    ];

    ProtocolBuilder::for_simulator()
        .configure_babylon(|_| BabylonSettings {
            genesis_data_chunks: validator_chunks,
            genesis_epoch: Epoch::of(1),
            consensus_manager_config: ConsensusManagerConfig::test_default(),
            initial_time_ms: 1,
            initial_current_leader: Some(0),
            faucet_supply: Decimal::zero(),
        })
        .only_babylon()
        .commit_each_protocol_update(&mut substate_db);

    let reader = SystemDatabaseReader::new(&substate_db);

    let validator_sort_key = reader
        .collection_iter(CONSENSUS_MANAGER.as_node_id(), ModuleId::Main, 0)
        .expect("collection not found")
        .map(|(key, _value)| key)
        .next()
        .expect("collection empty");

    let SubstateKey::Sorted((sort_prefix, address)) = validator_sort_key else {
        panic!("collection not a sorted index");
    };
    let address: ComponentAddress = scrypto_decode(address.as_slice()).expect("not an address");
    let validator = reader
        .read_object_collection_entry::<_, VersionedConsensusManagerRegisteredValidatorByStake>(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ObjectCollectionKey::SortedIndex(0, u16::from_be_bytes(sort_prefix), &address),
        )
        .expect("validator cannot be read")
        .map(|versioned| versioned.fully_update_and_into_latest_version())
        .expect("validator not found");

    assert_eq!(
        validator,
        Validator {
            key: validator_key,
            stake: stake_xrd,
        }
    );
}
