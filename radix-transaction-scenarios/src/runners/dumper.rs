use crate::internal_prelude::*;
use crate::scenarios::*;
use radix_engine::system::system_callback_api::SystemCallbackObject;
use radix_engine::updates::*;
use radix_engine::vm::{DefaultNativeVm, NativeVm, NoExtension, Vm};
use radix_engine::{
    system::bootstrap::Bootstrapper,
    vm::{
        wasm::{DefaultWasmEngine, WasmEngine},
        ScryptoVm,
    },
};
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_impls::state_tree_support::StateTreeUpdatingDatabase;
use radix_substate_store_interface::db_key_mapper::*;
use radix_substate_store_interface::interface::*;
use radix_transactions::validation::{NotarizedTransactionValidator, ValidationConfig};

pub struct RunnerContext {
    #[cfg(feature = "std")]
    pub dump_manifest_root: Option<std::path::PathBuf>,
    pub network: NetworkDefinition,
}

#[cfg(feature = "std")]
pub fn run_all_in_memory_and_dump_examples(
    network: NetworkDefinition,
    root_path: std::path::PathBuf,
) -> Result<(), FullScenarioError> {
    let mut event_hasher = HashAccumulator::new();
    let mut substate_db = StateTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());

    let ScenarioExecutionReceipt {
        database: mut substate_db,
    } = DefaultTransactionScenarioExecutor::new(substate_db, NetworkDefinition::simulator())
        .on_scenario_start(|scenario_metadata| {
            let sub_folder = root_path.join(scenario_metadata.logical_name);
            if sub_folder.exists() {
                std::fs::remove_dir_all(&sub_folder).unwrap();
            }
        })
        .on_transaction_executed(|scenario_metadata, transaction, receipt, _| {
            transaction.dump_manifest(
                &Some(root_path.join(scenario_metadata.logical_name)),
                &NetworkDefinition::simulator(),
            );

            let intent_hash =
                PreparedNotarizedTransactionV1::prepare_from_raw(&transaction.raw_transaction)
                    .unwrap()
                    .intent_hash();

            match &receipt.result {
                TransactionResult::Commit(c) => {
                    event_hasher.update_no_chain(intent_hash.as_hash().as_bytes());
                    event_hasher.update_no_chain(scrypto_encode(&c.application_events).unwrap());
                }
                TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
            }
        })
        .nonce_handling(ScenarioStartNonceHandling::PreviousScenarioStartNoncePlus(
            1000,
        ))
        .execute_all()
        .expect("Must succeed");

    assert_eq!(
        substate_db.get_current_root_hash().to_string(),
        "e53f921e3bda4d4f2ead1a049af443e913c5b66f3910e036b273361c4e2c6ae1"
    );
    assert_eq!(
        event_hasher.finalize().to_string(),
        "d80f0d118477e6417d723d4a301db6bdf25f09d2f252a454777e8d04e5cc6d49"
    );

    Ok(())
}

#[cfg(test)]
#[allow(irrefutable_let_patterns)]
mod test {
    use super::*;
    use radix_engine::vm::*;
    use radix_transactions::manifest::*;

    #[test]
    #[cfg(feature = "std")]
    pub fn update_expected_scenario_output() {
        let network_definition = NetworkDefinition::simulator();
        let scenarios_dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-examples");
        run_all_in_memory_and_dump_examples(network_definition.clone(), scenarios_dir.clone())
            .unwrap();

        // Ensure that they can all be compiled back again
        for entry in walkdir::WalkDir::new(&scenarios_dir) {
            let path = entry.unwrap().path().canonicalize().unwrap();
            if path.extension().and_then(|str| str.to_str()) != Some("rtm") {
                continue;
            }

            let manifest_string = std::fs::read_to_string(path).unwrap();
            compile(
                &manifest_string,
                &network_definition,
                MockBlobProvider::new(),
            )
            .unwrap();
        }
    }

    #[test]
    pub fn check_state_and_event_hashes_for_up_to_genesis_scenarios() {
        assert_event_and_state_hashes(
            "43be4cce2d4f2ed2eb519d77dfa770697244e843b2a0f7fd86bdf773d9b6f278",
            "1be7a3d32b165f77a2126e706ed1d79b9198a09a1f08fa8b0f168ed54e8a19cc",
            ScenarioFilter::AllValidBeforeProtocolVersion(Boundary::Exclusive(
                ProtocolVersion::ProtocolUpdate(ProtocolUpdate::Anemone),
            )),
            |_, _, _| {},
            |_, _| {},
        );
    }

    // No scenarios but ends with the protocol update which makes the root hash different but the
    // event hash the same as genesis.
    #[test]
    pub fn check_state_and_event_hashes_for_up_to_anemone_scenarios() {
        assert_event_and_state_hashes(
            "17567dbaf89a77a20e837e8d48187585b0547374fac9e19b9acc9d04d630a774",
            "1be7a3d32b165f77a2126e706ed1d79b9198a09a1f08fa8b0f168ed54e8a19cc",
            ScenarioFilter::AllValidBeforeProtocolVersion(Boundary::Inclusive(
                ProtocolVersion::ProtocolUpdate(ProtocolUpdate::Anemone),
            )),
            |network, protocol_update, db| {
                if let ProtocolVersion::ProtocolUpdate(protocol_update @ ProtocolUpdate::Anemone) =
                    protocol_update
                {
                    protocol_update
                        .generate_state_updates(db, network)
                        .into_iter()
                        .for_each(|update| {
                            db.commit(&update.create_database_updates::<SpreadPrefixKeyMapper>())
                        });
                }
            },
            |_, _| {},
        );
    }

    // Running the genesis scenarios against the anemone protocol update. We expect the state root
    // hash and the event hash to differ since the behavior of some blueprints changed in that
    // update.
    #[test]
    pub fn check_state_and_event_hashes_for_up_to_genesis_scenarios_on_anemone() {
        assert_event_and_state_hashes(
            "dbba9b7154eb40b3978a4d4b7921945178ee93f3bb04aad4ab08db98287a7785",
            "57e3e4d7a7232612540d949dcea540f7ead064a92d5da50fbe60b5659a2ddd0b",
            ScenarioFilter::AllValidBeforeProtocolVersion(Boundary::Inclusive(
                ProtocolVersion::ProtocolUpdate(ProtocolUpdate::Anemone),
            )),
            |_, _, _| {},
            // Update to anemone immediately after bootstrapping.
            |network, db| {
                ProtocolUpdate::Anemone
                    .generate_state_updates(db, network)
                    .into_iter()
                    .for_each(|update| {
                        db.commit(&update.create_database_updates::<SpreadPrefixKeyMapper>())
                    })
            },
        );
    }

    #[test]
    pub fn check_state_and_event_hashes_for_up_to_bottlenose_scenarios() {
        assert_event_and_state_hashes(
            "e53f921e3bda4d4f2ead1a049af443e913c5b66f3910e036b273361c4e2c6ae1",
            "d80f0d118477e6417d723d4a301db6bdf25f09d2f252a454777e8d04e5cc6d49",
            ScenarioFilter::AllValidBeforeProtocolVersion(Boundary::Inclusive(
                ProtocolVersion::ProtocolUpdate(ProtocolUpdate::Bottlenose),
            )),
            |network, protocol_update, db| {
                if let ProtocolVersion::ProtocolUpdate(
                    protocol_update @ (ProtocolUpdate::Anemone | ProtocolUpdate::Bottlenose),
                ) = protocol_update
                {
                    protocol_update
                        .generate_state_updates(db, network)
                        .into_iter()
                        .for_each(|update| {
                            db.commit(&update.create_database_updates::<SpreadPrefixKeyMapper>())
                        });
                }
            },
            |_, _| {},
        );
    }

    #[test]
    pub fn check_state_and_event_hashes_for_up_to_anemone_scenarios_on_bottlenose() {
        assert_event_and_state_hashes(
            "9d06891ba213a57e14031af89cd21b040a1e0c7b362f5e425d1dd281e258d47d",
            "b28281fc7fa97a500aca0e381547697cde840b3e0815f4911f529dec1b8f4f41",
            ScenarioFilter::AllValidBeforeProtocolVersion(Boundary::Inclusive(
                ProtocolVersion::ProtocolUpdate(ProtocolUpdate::Anemone),
            )),
            |_, _, _| {},
            // Update to bottlenose immediately after bootstrapping.
            |network, db| {
                [ProtocolUpdate::Anemone, ProtocolUpdate::Bottlenose]
                    .into_iter()
                    .for_each(|protocol_update| {
                        protocol_update
                            .generate_state_updates(db, network)
                            .into_iter()
                            .for_each(|update| {
                                db.commit(
                                    &update.create_database_updates::<SpreadPrefixKeyMapper>(),
                                )
                            })
                    })
            },
        );
    }

    #[test]
    pub fn check_state_and_event_hashes_for_up_to_genesis_scenarios_on_bottlenose() {
        assert_event_and_state_hashes(
            "9d06891ba213a57e14031af89cd21b040a1e0c7b362f5e425d1dd281e258d47d",
            "b28281fc7fa97a500aca0e381547697cde840b3e0815f4911f529dec1b8f4f41",
            ScenarioFilter::AllValidBeforeProtocolVersion(Boundary::Exclusive(
                ProtocolVersion::ProtocolUpdate(ProtocolUpdate::Anemone),
            )),
            |_, _, _| {},
            // Update to bottlenose immediately after bootstrapping.
            |network, db| {
                [ProtocolUpdate::Anemone, ProtocolUpdate::Bottlenose]
                    .into_iter()
                    .for_each(|protocol_update| {
                        protocol_update
                            .generate_state_updates(db, network)
                            .into_iter()
                            .for_each(|update| {
                                db.commit(
                                    &update.create_database_updates::<SpreadPrefixKeyMapper>(),
                                )
                            })
                    })
            },
        );
    }

    fn assert_event_and_state_hashes<P, B>(
        expected_state_root_hash: &str,
        expected_event_hash: &str,
        filter: ScenarioFilter,
        protocol_update_handling: P,
        after_bootstrap: B,
    ) where
        P: FnMut(
            &NetworkDefinition,
            ProtocolVersion,
            &mut StateTreeUpdatingDatabase<InMemorySubstateDatabase>,
        ),
        B: FnMut(&NetworkDefinition, &mut StateTreeUpdatingDatabase<InMemorySubstateDatabase>),
    {
        // Arrange
        let mut event_hasher = HashAccumulator::new();
        let mut substate_db = StateTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());

        // Act
        let ScenarioExecutionReceipt {
            database: mut substate_db,
        } = DefaultTransactionScenarioExecutor::new(substate_db, NetworkDefinition::simulator())
            .on_transaction_executed(|metadata, transaction, receipt, _| {
                let intent_hash =
                    PreparedNotarizedTransactionV1::prepare_from_raw(&transaction.raw_transaction)
                        .unwrap()
                        .intent_hash();

                match &receipt.result {
                    TransactionResult::Commit(c) => {
                        event_hasher.update_no_chain(intent_hash.as_hash().as_bytes());
                        event_hasher
                            .update_no_chain(scrypto_encode(&c.application_events).unwrap());
                    }
                    TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
                }
            })
            .on_new_protocol_requirement_encountered(protocol_update_handling)
            .after_bootstrap(after_bootstrap)
            .nonce_handling(ScenarioStartNonceHandling::PreviousScenarioStartNoncePlus(
                1000,
            ))
            .execute_all_matching(filter)
            .expect("Must succeed");

        // Assert
        assert_eq!(
            substate_db.get_current_root_hash().to_string(),
            expected_state_root_hash
        );
        assert_eq!(event_hasher.finalize().to_string(), expected_event_hash);
    }
}
