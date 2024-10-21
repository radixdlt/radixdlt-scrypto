use radix_engine::transaction::*;
use radix_engine::updates::*;
use radix_engine::vm::NoExtension;
use radix_engine::vm::VmModules;
use radix_substate_store_impls::memory_db::*;
use radix_substate_store_impls::substate_database_overlay::*;
use radix_substate_store_interface::interface::*;
use radix_transaction_scenarios::executor::*;
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
    let substate = db.get_raw_substate_by_db_key(
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
    let substate = db.get_raw_substate_by_db_key(
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
    let substate = db.get_raw_substate_by_db_key(
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
    let substate = db.get_raw_substate_by_db_key(
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
    let substate = db.get_raw_substate_by_db_key(
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
    let substate = db.get_raw_substate_by_db_key(
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
    let mut substates = db.list_raw_values_from_db_key(
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
    let mut substates = db.list_raw_values_from_db_key(
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
        .build();

    let (public_key1, _, account1) = ledger.new_account(false);
    let (public_key2, _, account2) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account1, XRD, dec!(100))
            .deposit_entire_worktop(account2)
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
    struct Check;
    impl DatabaseComparisonScenarioCheck for Check {
        fn check(
            &mut self,
            _in_memory_database: &InMemorySubstateDatabase,
            in_memory_receipt: &TransactionReceipt,
            _overlay_database: &UnmergeableSubstateDatabaseOverlay<'_, InMemorySubstateDatabase>,
            overlay_receipt: &TransactionReceipt,
        ) {
            assert_eq!(in_memory_receipt, overlay_receipt)
        }
    }
    run_scenarios_in_memory_and_on_overlay(Check);
}

#[test]
#[allow(clippy::redundant_closure_call)]
fn database_hashes_are_identical_between_staging_and_non_staging_database_at_each_scenario_step() {
    struct Check;
    impl DatabaseComparisonScenarioCheck for Check {
        fn check(
            &mut self,
            in_memory_database: &InMemorySubstateDatabase,
            _in_memory_receipt: &TransactionReceipt,
            overlay_database: &UnmergeableSubstateDatabaseOverlay<'_, InMemorySubstateDatabase>,
            _overlay_receipt: &TransactionReceipt,
        ) {
            let non_staging_database_hash = create_database_contents_hash(in_memory_database);
            let staging_database_hash = create_database_contents_hash(overlay_database);

            assert_eq!(non_staging_database_hash, staging_database_hash)
        }
    }
    run_scenarios_in_memory_and_on_overlay(Check);
}

fn create_database_contents_hash<D: SubstateDatabase + ListableSubstateDatabase>(
    database: &D,
) -> Hash {
    let mut accumulator_hash = Hash([0; 32]);
    for (node_id, partition_number) in database.read_partition_keys() {
        for (substate_key, substate_value) in
            database.list_raw_values(node_id, partition_number, None::<SubstateKey>)
        {
            let entry_hash = hash(
                scrypto_encode(&(node_id, partition_number, substate_key, substate_value)).unwrap(),
            );
            let mut data = accumulator_hash.to_vec();
            data.extend(entry_hash.to_vec());
            accumulator_hash = hash(data);
        }
    }
    accumulator_hash
}

trait DatabaseComparisonScenarioCheck {
    fn check(
        &mut self,
        in_memory_database: &InMemorySubstateDatabase,
        in_memory_receipt: &TransactionReceipt,
        overlay_database: &UnmergeableSubstateDatabaseOverlay<'_, InMemorySubstateDatabase>,
        overlay_receipt: &TransactionReceipt,
    );
}

/// Runs the scenarios on an [`InMemorySubstateDatabase`] and a [`UnmergeableSubstateDatabaseOverlay`] wrapping
/// an [`InMemorySubstateDatabase`]. The passed check function is executed after the execution of
/// each scenario.
fn run_scenarios_in_memory_and_on_overlay(check_callback: impl DatabaseComparisonScenarioCheck) {
    let overlay_root = InMemorySubstateDatabase::standard();
    let overlay = SubstateDatabaseOverlay::new_unmergeable(&overlay_root);
    let ledger_with_overlay = Rc::new(RefCell::new(
        LedgerSimulatorBuilder::new()
            .with_custom_database(overlay)
            .with_custom_protocol(|builder| builder.unbootstrapped())
            .build(),
    ));
    let network_definition = NetworkDefinition::simulator();

    struct ProtocolUpdateHooks<'a> {
        ledger_with_overlay: Rc<
            RefCell<
                LedgerSimulator<
                    NoExtension,
                    SubstateDatabaseOverlay<&'a InMemorySubstateDatabase, InMemorySubstateDatabase>,
                >,
            >,
        >,
    }

    impl<'a> ProtocolUpdateExecutionHooks for ProtocolUpdateHooks<'a> {
        fn on_transaction_executed(&mut self, event: OnProtocolTransactionExecuted) {
            let OnProtocolTransactionExecuted { receipt, .. } = event;
            // We copy the protocol updates onto the ledger_with_overlay
            let database_updates = receipt
                .expect_commit_success()
                .state_updates
                .create_database_updates();
            self.ledger_with_overlay
                .borrow_mut()
                .substate_db_mut()
                .commit(&database_updates);
        }

        fn on_transaction_batch_committed(&mut self, event: OnProtocolTransactionBatchCommitted) {
            let OnProtocolTransactionBatchCommitted {
                status_update_committed,
                protocol_version,
                batch_group_index,
                batch_group_name,
                batch_index,
                batch_name,
                ..
            } = event;
            if status_update_committed {
                self.ledger_with_overlay
                    .borrow_mut()
                    .substate_db_mut()
                    .update_substate(
                        TRANSACTION_TRACKER,
                        PROTOCOL_UPDATE_STATUS_PARTITION,
                        ProtocolUpdateStatusField::Summary,
                        ProtocolUpdateStatusSummarySubstate::from_latest_version(
                            ProtocolUpdateStatusSummaryV1 {
                                protocol_version: protocol_version,
                                update_status: ProtocolUpdateStatus::InProgress {
                                    latest_commit: LatestProtocolUpdateCommitBatch {
                                        batch_group_index,
                                        batch_group_name: batch_group_name.to_string(),
                                        batch_index,
                                        batch_name: batch_name.to_string(),
                                    },
                                },
                            },
                        ),
                    );
            }
        }

        fn on_protocol_update_completed(&mut self, event: OnProtocolUpdateCompleted) {
            let OnProtocolUpdateCompleted {
                protocol_version,
                status_update_committed,
                ..
            } = event;
            if status_update_committed {
                self.ledger_with_overlay
                    .borrow_mut()
                    .substate_db_mut()
                    .update_substate(
                        TRANSACTION_TRACKER,
                        PROTOCOL_UPDATE_STATUS_PARTITION,
                        ProtocolUpdateStatusField::Summary,
                        ProtocolUpdateStatusSummarySubstate::from_latest_version(
                            ProtocolUpdateStatusSummaryV1 {
                                protocol_version: protocol_version,
                                update_status: ProtocolUpdateStatus::Complete,
                            },
                        ),
                    );
                self.ledger_with_overlay
                    .borrow_mut()
                    .update_transaction_validator_after_manual_protocol_update();
            }
        }
    }

    struct ScenarioHooks<'a, F: DatabaseComparisonScenarioCheck> {
        ledger_with_overlay: Rc<
            RefCell<
                LedgerSimulator<
                    NoExtension,
                    SubstateDatabaseOverlay<&'a InMemorySubstateDatabase, InMemorySubstateDatabase>,
                >,
            >,
        >,
        check_callback: F,
    }

    impl<'a, F: DatabaseComparisonScenarioCheck> ScenarioExecutionHooks<InMemorySubstateDatabase>
        for ScenarioHooks<'a, F>
    {
        fn on_transaction_executed(
            &mut self,
            event: OnScenarioTransactionExecuted<InMemorySubstateDatabase>,
        ) {
            let OnScenarioTransactionExecuted {
                transaction,
                receipt,
                database,
                ..
            } = event;

            // Execute the same transaction on the ledger simulator.
            let receipt_from_overlay = self
                .ledger_with_overlay
                .borrow_mut()
                .execute_notarized_transaction(&transaction.raw_transaction);

            // Check that everything matches.
            self.check_callback.check(
                database,
                &receipt,
                self.ledger_with_overlay.borrow().substate_db(),
                &receipt_from_overlay,
            );
        }
    }

    TransactionScenarioExecutor::new(InMemorySubstateDatabase::standard(), network_definition)
        .execute_protocol_updates_and_scenarios(
            |builder| builder.from_bootstrap_to_latest(),
            ScenarioTrigger::AtStartOfEveryProtocolVersion,
            ScenarioFilter::AllScenariosFirstValidAtProtocolVersion,
            &mut ScenarioHooks {
                ledger_with_overlay: ledger_with_overlay.clone(),
                check_callback,
            },
            &mut ProtocolUpdateHooks {
                ledger_with_overlay: ledger_with_overlay.clone(),
            },
            &VmModules::default(),
        )
        .expect("Must succeed!");
}
