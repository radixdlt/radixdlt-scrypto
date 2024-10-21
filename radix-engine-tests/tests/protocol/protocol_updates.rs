use scrypto_test::prelude::*;

#[test]
fn protocol_updates_can_be_continued_from_any_complete_update() {
    // This demonstrates that `from_current_to_latest` works correctly, and
    // can be used by e.g. the simulator environment to migrate after a protocol
    // update
    for stop_after_update in ProtocolVersion::all_from(ProtocolVersion::Unbootstrapped) {
        let mut simulator = LedgerSimulatorBuilder::new()
            .with_custom_protocol(|builder| builder.from_bootstrap_to(stop_after_update))
            .build();

        let mut record_first_batch_hooks = RecordFirstBatchHooks { first_batch: None };
        ProtocolBuilder::for_simulator()
            .from_current_to_latest()
            .commit_each_protocol_update_advanced(
                simulator.substate_db_mut(),
                &mut record_first_batch_hooks,
                &DefaultVmModules::default(),
            );

        let expected_next_update_batch = stop_after_update.next().map(|version| (version, 0, 0));

        assert_eq!(
            record_first_batch_hooks.first_batch,
            expected_next_update_batch,
        );

        assert_at_latest_version(simulator.substate_db());
    }
}

#[test]
#[cfg(feature = "std")]
fn protocol_updates_can_be_resumed_between_batches_from_cuttlefish() {
    use std::panic;

    // STEP 1
    // We run protocol updates but abort in cuttlefish
    let stop_after_batch = (ProtocolVersion::Cuttlefish, 0, 0);

    struct HaltMidUpdateHooks {
        stop_after_batch: (ProtocolVersion, usize, usize),
    }
    impl ProtocolUpdateExecutionHooks for HaltMidUpdateHooks {
        fn on_transaction_batch_committed(&mut self, event: OnProtocolTransactionBatchCommitted) {
            let OnProtocolTransactionBatchCommitted {
                protocol_version,
                batch_group_index,
                batch_index,
                ..
            } = event;
            let stop_after_batch = self.stop_after_batch;
            if protocol_version == stop_after_batch.0
                && batch_group_index == stop_after_batch.1
                && batch_index == stop_after_batch.2
            {
                panic!("ABORTED MID-UPDATE");
            }
        }
    }
    let mut simulator = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.unbootstrapped())
        .build();

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        ProtocolBuilder::for_simulator()
            .from_current_to_latest()
            .commit_each_protocol_update_advanced(
                simulator.substate_db_mut(),
                &mut HaltMidUpdateHooks { stop_after_batch },
                &DefaultVmModules::default(),
            );
    }));
    let panic_string = *result
        .expect_err("Expected a panic")
        .downcast_ref::<&str>()
        .unwrap();
    assert_eq!(panic_string, "ABORTED MID-UPDATE");

    let substate: ProtocolUpdateStatusSummarySubstate =
        simulator.substate_db().get_existing_substate(
            TRANSACTION_TRACKER,
            PROTOCOL_UPDATE_STATUS_PARTITION,
            ProtocolUpdateStatusField::Summary,
        );
    let summary = substate.into_unique_version();
    assert_eq!(summary.protocol_version, stop_after_batch.0);
    assert_eq!(
        summary.update_status,
        ProtocolUpdateStatus::InProgress {
            latest_commit: LatestProtocolUpdateCommitBatch {
                batch_group_index: stop_after_batch.1,
                batch_group_name: "principal".to_string(),
                batch_index: stop_after_batch.2,
                batch_name: "primary".to_string(),
            },
        }
    );

    let mut record_first_batch_hooks = RecordFirstBatchHooks { first_batch: None };
    ProtocolBuilder::for_simulator()
        .from_current_to_latest()
        .commit_each_protocol_update_advanced(
            simulator.substate_db_mut(),
            &mut record_first_batch_hooks,
            &DefaultVmModules::default(),
        );

    match record_first_batch_hooks.first_batch {
        Some(next_batch) => {
            assert!(next_batch > stop_after_batch);
            let is_protocol_version_successor = Some(next_batch.0) == stop_after_batch.0.next()
                && next_batch.1 == 0
                && next_batch.2 == 0;
            let is_batch_group_successor = next_batch.0 == stop_after_batch.0
                && next_batch.1 == stop_after_batch.1 + 1
                && next_batch.2 == 0;
            let is_batch_successor = next_batch.0 == stop_after_batch.0
                && next_batch.1 == stop_after_batch.1
                && next_batch.2 == stop_after_batch.2 + 1;
            if !(is_protocol_version_successor || is_batch_group_successor || is_batch_successor) {
                panic!(
                    "The next protocol update batch is not a direct successor to the halted batch"
                );
            }
        }
        None => {
            // If there aren't any future batches, there's nothing to test on
        }
    }

    assert_at_latest_version(simulator.substate_db());
}

struct RecordFirstBatchHooks {
    first_batch: Option<(ProtocolVersion, usize, usize)>,
}

impl ProtocolUpdateExecutionHooks for RecordFirstBatchHooks {
    fn on_transaction_batch_committed(&mut self, event: OnProtocolTransactionBatchCommitted) {
        if self.first_batch.is_some() {
            return;
        }
        let OnProtocolTransactionBatchCommitted {
            protocol_version,
            batch_group_index,
            batch_index,
            ..
        } = event;
        self.first_batch = Some((protocol_version, batch_group_index, batch_index));
    }
}

fn assert_at_latest_version(database: &impl SubstateDatabase) {
    let substate: ProtocolUpdateStatusSummarySubstate = database.get_existing_substate(
        TRANSACTION_TRACKER,
        PROTOCOL_UPDATE_STATUS_PARTITION,
        ProtocolUpdateStatusField::Summary,
    );
    let summary = substate.into_unique_version();
    assert_eq!(summary.protocol_version, ProtocolVersion::LATEST);
    assert_eq!(summary.update_status, ProtocolUpdateStatus::Complete);
}
