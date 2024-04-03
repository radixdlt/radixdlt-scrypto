use radix_engine::system::system_db_reader::*;
use radix_engine::transaction::*;
use radix_substate_store_impls::memory_db::*;
use radix_substate_store_impls::substate_database_overlay::*;
use radix_substate_store_interface::db_key_mapper::*;
use radix_substate_store_interface::interface::*;
use radix_transaction_scenarios::scenario::*;
use radix_transaction_scenarios::scenarios::*;
use radix_transactions::builder::*;
use scrypto::prelude::*;
use scrypto_test::ledger_simulator::*;

#[test]
fn substates_written_to_root_database_can_be_read() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let db = SubstateDatabaseOverlay::new_unmergeable(&root);

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, Some(b"some-substate-value".to_vec()))
}

#[test]
fn substates_written_to_overlay_can_be_read_later() {
    // Arrange
    let root = InMemorySubstateDatabase::standard();
    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);

    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, Some(b"some-substate-value".to_vec()))
}

#[test]
fn substate_deletes_to_overlay_prevent_substate_from_being_read() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Delete
                        }
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, None)
}

#[test]
fn partition_deletes_to_overlay_prevent_substate_from_being_read() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{}
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, None)
}

#[test]
fn partition_resets_to_overlay_return_new_substate_data() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{
                            DbSortKey(b"some-sort-key".to_vec()) => b"some-other-value".to_vec()
                        }
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, Some(b"some-other-value".to_vec()))
}

#[test]
fn partition_resets_are_not_combined() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{
                            DbSortKey(b"some-sort-key".to_vec()) => b"some-other-value".to_vec()
                        }
                    }
                }
            }
        },
    });
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{}
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, None)
}

#[test]
fn from_sort_key_in_list_entries_from_works_when_the_overlay_is_in_reset_mode() {
    // Arrange
    let root = InMemorySubstateDatabase::standard();
    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);

    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{
                            DbSortKey([0].to_vec()) => b"0".to_vec(),
                            DbSortKey([1].to_vec()) => b"1".to_vec(),
                            DbSortKey([2].to_vec()) => b"2".to_vec()
                        }
                    }
                }
            }
        },
    });

    // Act
    let mut substates = db.list_entries_from(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        Some(&DbSortKey([1].to_vec())),
    );

    // Assert
    let substate1 = substates.next().expect("We must get the first substate");
    let substate2 = substates.next().expect("We must get the first substate");
    assert_eq!(
        substates.next(),
        None,
        "Another substate is available after the two substates"
    );

    assert_eq!(substate1, (DbSortKey([1].to_vec()), b"1".to_vec()));
    assert_eq!(substate2, (DbSortKey([2].to_vec()), b"2".to_vec()));
}

#[test]
fn from_sort_key_in_list_entries_from_works_when_the_overlay_is_in_delta_mode() {
    // Arrange
    let root = InMemorySubstateDatabase::standard();
    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);

    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap!{
                            DbSortKey([0].to_vec()) => DatabaseUpdate::Set(b"0".to_vec()),
                            DbSortKey([1].to_vec()) => DatabaseUpdate::Set(b"1".to_vec()),
                            DbSortKey([2].to_vec()) => DatabaseUpdate::Set(b"2".to_vec())
                        }
                    }
                }
            }
        },
    });

    // Act
    let mut substates = db.list_entries_from(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        Some(&DbSortKey([1].to_vec())),
    );

    // Assert
    let substate1 = substates.next().expect("We must get the first substate");
    let substate2 = substates.next().expect("We must get the first substate");
    assert_eq!(
        substates.next(),
        None,
        "Another substate is available after the two substates"
    );

    assert_eq!(substate1, (DbSortKey([1].to_vec()), b"1".to_vec()));
    assert_eq!(substate2, (DbSortKey([2].to_vec()), b"2".to_vec()));
}

#[test]
fn substates_written_on_a_staging_database_from_transactions_can_be_read_later() {
    // Arrange
    let root_database = InMemorySubstateDatabase::standard();
    let database = SubstateDatabaseOverlay::new_unmergeable(&root_database);
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_database(database)
        .without_kernel_trace()
        .build();

    let (public_key1, _, account1) = ledger.new_account(false);
    let (public_key2, _, account2) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account1, XRD, dec!(100))
            .deposit_batch(account2)
            .build(),
        [public_key1, public_key2]
            .map(|pk| NonFungibleGlobalId::from_public_key(&pk))
            .to_vec(),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn transaction_receipts_from_scenarios_are_identical_between_staging_and_non_staging_database() {
    run_scenarios(|(_, non_staging_receipt), (_, staging_receipt)| {
        assert_eq!(non_staging_receipt, staging_receipt)
    })
}

#[test]
#[allow(clippy::redundant_closure_call)]
fn database_hashes_are_identical_between_staging_and_non_staging_database_at_each_scenario_step() {
    macro_rules! non_homogenous_array_map {
        (
            [
                $($item: expr),* $(,)?
            ]
            .map($func: expr)
        ) => {
            [
                $(
                    $func($item)
                ),*
            ]
        };
    }

    run_scenarios(|(non_staging_database, _), (staging_database, _)| {
        let [non_staging_database_hash, staging_database_hash] = non_homogenous_array_map! {
            [non_staging_database, staging_database].map(|database| {
                let mut accumulator_hash = Hash([0; 32]);
                let reader = SystemDatabaseReader::new(database);
                for (node_id, partition_number) in reader.partitions_iter() {
                    let db_node_key = SpreadPrefixKeyMapper::to_db_node_key(&node_id);
                    let db_partition_key = DbPartitionKey {
                        node_key: db_node_key,
                        partition_num: partition_number.0,
                    };

                    for (substate_key, substate_value) in
                        SubstateDatabase::list_entries(database, &db_partition_key)
                    {
                        let entry_hash = hash(
                            scrypto_encode(&(node_id, partition_number, substate_key, substate_value))
                                .unwrap(),
                        );
                        let mut data = accumulator_hash.to_vec();
                        data.extend(entry_hash.to_vec());
                        accumulator_hash = hash(data);
                    }
                }
                accumulator_hash
            })
        };

        assert_eq!(non_staging_database_hash, staging_database_hash)
    })
}

/// Runs the scenarios on an [`InMemorySubstateDatabase`] and a [`SingleSubstateDatabaseOverlay`] wrapping
/// an [`InMemorySubstateDatabase`]. The passed check function is executed after the execution of
/// each scenario.
fn run_scenarios(
    check_callback: impl Fn(
        (&InMemorySubstateDatabase, &TransactionReceipt),
        (
            &UnmergeableSubstateDatabaseOverlay<'_, InMemorySubstateDatabase>,
            &TransactionReceipt,
        ),
    ),
) {
    let network = NetworkDefinition::simulator();

    let db1 = InMemorySubstateDatabase::standard();

    let db2_root = InMemorySubstateDatabase::standard();
    let db2 = SubstateDatabaseOverlay::new_unmergeable(&db2_root);

    let mut ledger1 = LedgerSimulatorBuilder::new()
        .with_custom_database(db1)
        .without_kernel_trace()
        .build();
    let mut ledger2 = LedgerSimulatorBuilder::new()
        .with_custom_database(db2)
        .without_kernel_trace()
        .build();

    let mut next_nonce: u32 = 0;
    let mut scenario_builders = scenario_builders();
    for requirement in ScenarioRequirements::all() {
        if let ScenarioRequirements::ProtocolUpdateUpTo(protocol_update) = requirement {
            protocol_update
                .generate_state_updates(ledger1.substate_db_mut(), &network)
                .into_iter()
                .for_each(|state_updates| {
                    ledger1
                        .substate_db_mut()
                        .commit(&state_updates.create_database_updates::<SpreadPrefixKeyMapper>())
                });
            protocol_update
                .generate_state_updates(ledger2.substate_db_mut(), &network)
                .into_iter()
                .for_each(|state_updates| {
                    ledger2
                        .substate_db_mut()
                        .commit(&state_updates.create_database_updates::<SpreadPrefixKeyMapper>())
                })
        };

        let scenario_builders = scenario_builders
            .shift_remove(requirement)
            .unwrap_or_default();
        for scenario_builder in scenario_builders {
            let epoch = {
                assert_eq!(ledger1.get_current_epoch(), ledger2.get_current_epoch());
                ledger1.get_current_epoch()
            };
            let mut scenario =
                scenario_builder(ScenarioCore::new(network.clone(), epoch, next_nonce));
            let mut previous = None;
            loop {
                let next = scenario
                    .next(previous.as_ref())
                    .map_err(|err| err.into_full(&scenario))
                    .unwrap();
                match next {
                    NextAction::Transaction(next) => {
                        let receipt1 = ledger1.execute_notarized_transaction(&next.raw_transaction);
                        let receipt2 = ledger2.execute_notarized_transaction(&next.raw_transaction);

                        check_callback(
                            (ledger1.substate_db(), &receipt1),
                            (ledger2.substate_db(), &receipt2),
                        );

                        previous = Some(receipt1);
                    }
                    NextAction::Completed(end_state) => {
                        next_nonce = end_state.next_unused_nonce;
                        break;
                    }
                }
            }
        }
    }
}
