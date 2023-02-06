#[cfg(not(feature = "alloc"))]
mod multi_threaded_test {
    use radix_engine::kernel::ScryptoInterpreter;
    use radix_engine::ledger::*;
    use radix_engine::transaction::{execute_and_commit_transaction, execute_transaction};
    use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
    use radix_engine::types::*;
    use radix_engine::wasm::WasmInstrumenter;
    use radix_engine::wasm::{DefaultWasmEngine, WasmMeteringConfig};
    use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
    use radix_engine_interface::blueprints::resource::*;
    use radix_engine_interface::dec;
    use radix_engine_interface::rule;
    use transaction::builder::ManifestBuilder;
    use transaction::model::TestTransaction;
    use transaction::signing::EcdsaSecp256k1PrivateKey;
    // using crossbeam for its scoped thread feature, which allows non-static lifetimes for data being
    // passed to the thread (see https://docs.rs/crossbeam/0.8.2/crossbeam/thread/struct.Scope.html)
    extern crate crossbeam;
    use crossbeam::thread;

    // this test was inspired by radix_engine "Transfer" benchmark
    #[test]
    fn test_multithread_transfer() {
        // Set up environment.
        let mut scrypto_interpreter = ScryptoInterpreter {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_metering_config: WasmMeteringConfig::V0,
        };
        let mut substate_store = TypedInMemorySubstateStore::with_bootstrap(&scrypto_interpreter);

        // Create a key pair
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
        let public_key = private_key.public_key();

        // Create two accounts
        let accounts = (0..2)
            .map(|_| {
                let manifest = ManifestBuilder::new()
                    .lock_fee(FAUCET_COMPONENT, 100.into())
                    .new_account(&rule!(require(NonFungibleGlobalId::from_public_key(
                        &public_key
                    ))))
                    .build();
                let account = execute_and_commit_transaction(
                    &mut substate_store,
                    &mut scrypto_interpreter,
                    &FeeReserveConfig::default(),
                    &ExecutionConfig::default(),
                    &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                        .get_executable(vec![NonFungibleGlobalId::from_public_key(&public_key)]),
                )
                .expect_commit()
                .entity_changes
                .new_component_addresses[0];

                let manifest = ManifestBuilder::new()
                    .lock_fee(FAUCET_COMPONENT, 100.into())
                    .call_method(FAUCET_COMPONENT, "free", args!())
                    .call_method(
                        account,
                        "deposit_batch",
                        args!(ManifestExpression::EntireWorktop),
                    )
                    .build();
                execute_and_commit_transaction(
                    &mut substate_store,
                    &mut scrypto_interpreter,
                    &FeeReserveConfig::default(),
                    &ExecutionConfig::default(),
                    &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                        .get_executable(vec![NonFungibleGlobalId::from_public_key(&public_key)]),
                )
                .expect_commit();

                account
            })
            .collect::<Vec<ComponentAddress>>();

        let account1 = accounts[0];
        let account2 = accounts[1];

        // Fill first account
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .call_method(FAUCET_COMPONENT, "free", args!())
            .call_method(
                account1,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build();
        for nonce in 0..10 {
            execute_and_commit_transaction(
                &mut substate_store,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(vec![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit();
        }

        // Create a transfer manifest
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .withdraw_from_account(account1, RADIX_TOKEN, dec!("0.000001"))
            .call_method(
                account2,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build();

        let nonce = 3;

        // Spawning threads that will attempt to withdraw some XRD amount from account1 and deposit to
        // account2
        thread::scope(|s| {
            for _i in 0..20 {
                s.spawn(|_| {
                    let receipt = execute_transaction(
                        &substate_store,
                        &scrypto_interpreter,
                        &FeeReserveConfig::default(),
                        &ExecutionConfig::default(),
                        &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                            .get_executable(vec![NonFungibleGlobalId::from_public_key(
                                &public_key,
                            )]),
                    );
                    receipt.expect_commit_success();
                    println!("recept = {:?}", receipt);
                });
            }
        })
        .unwrap();
    }
}
