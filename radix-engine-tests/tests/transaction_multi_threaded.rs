#[cfg(not(feature = "alloc"))]
mod multi_threaded_test {
    use radix_engine::system::bootstrap::Bootstrapper;
    use radix_engine::transaction::{execute_and_commit_transaction, execute_transaction};
    use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
    use radix_engine::types::*;
    use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
    use radix_engine_interface::dec;
    use radix_engine_interface::rule;
    use radix_engine_stores::memory_db::InMemorySubstateDatabase;
    use transaction::builder::ManifestBuilder;
    use transaction::model::TestTransaction;
    use transaction::signing::secp256k1::Secp256k1PrivateKey;
    // using crossbeam for its scoped thread feature, which allows non-static lifetimes for data being
    // passed to the thread (see https://docs.rs/crossbeam/0.8.2/crossbeam/thread/struct.Scope.html)
    extern crate crossbeam;
    use crossbeam::thread;
    use radix_engine::vm::ScryptoVm;

    // this test was inspired by radix_engine "Transfer" benchmark
    #[test]
    fn test_multithread_transfer() {
        // Set up environment.
        let mut scrypto_interpreter = ScryptoVm {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_validator_config: WasmValidatorConfigV1::new(),
        };
        let mut substate_db = InMemorySubstateDatabase::standard();
        Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false)
            .bootstrap_test_default()
            .unwrap();

        // Create a key pair
        let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let public_key = private_key.public_key();

        // Create two accounts
        let accounts = (0..2)
            .map(|i| {
                let manifest = ManifestBuilder::new()
                    .lock_fee(FAUCET, 500u32.into())
                    .new_account_advanced(OwnerRole::Fixed(rule!(require(
                        NonFungibleGlobalId::from_public_key(&public_key)
                    ))))
                    .build();
                let account = execute_and_commit_transaction(
                    &mut substate_db,
                    &mut scrypto_interpreter,
                    &FeeReserveConfig::default(),
                    &ExecutionConfig::for_test_transaction(),
                    &TestTransaction::new(manifest.clone(), hash(format!("Account creation: {i}")))
                        .prepare()
                        .unwrap()
                        .get_executable(btreeset![NonFungibleGlobalId::from_public_key(
                            &public_key
                        )]),
                )
                .expect_commit(true)
                .new_component_addresses()[0];
                account
            })
            .collect::<Vec<ComponentAddress>>();

        let account1 = accounts[0];
        let account2 = accounts[1];

        // Fill first account
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 500u32.into())
            .call_method(FAUCET, "free", manifest_args!())
            .call_method(
                account1,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        for nonce in 0..10 {
            execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::for_test_transaction(),
                &TestTransaction::new(manifest.clone(), hash(format!("Fill account: {}", nonce)))
                    .prepare()
                    .expect("Expected transaction to be preparable")
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit(true);
        }

        // Create a transfer manifest
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 500u32.into())
            .withdraw_from_account(account1, RADIX_TOKEN, dec!("0.000001"))
            .call_method(
                account2,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();

        // Spawning threads that will attempt to withdraw some XRD amount from account1 and deposit to
        // account2
        thread::scope(|s| {
            for _i in 0..20 {
                // Note - we run the same transaction on all threads, but don't commit anything
                s.spawn(|_| {
                    let receipt = execute_transaction(
                        &substate_db,
                        &scrypto_interpreter,
                        &FeeReserveConfig::default(),
                        &ExecutionConfig::for_test_transaction(),
                        &TestTransaction::new(manifest.clone(), hash(format!("Transfer")))
                            .prepare()
                            .expect("Expected transaction to be preparable")
                            .get_executable(btreeset![NonFungibleGlobalId::from_public_key(
                                &public_key,
                            )]),
                    );
                    receipt.expect_commit_success();
                    println!("receipt = {:?}", receipt);
                });
            }
        })
        .unwrap();
    }
}
