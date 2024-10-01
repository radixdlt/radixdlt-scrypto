#[cfg(not(feature = "alloc"))]
#[cfg(not(feature = "resource_tracker"))]
mod multi_threaded_test {
    use radix_common::prelude::*;
    use radix_engine::transaction::ExecutionConfig;
    use radix_engine::transaction::{execute_and_commit_transaction, execute_transaction};
    use radix_engine::updates::ProtocolBuilder;
    use radix_engine_interface::prelude::*;
    use radix_engine_interface::rule;
    use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
    use radix_transactions::model::TestTransaction;
    use radix_transactions::prelude::*;
    // using crossbeam for its scoped thread feature, which allows non-static lifetimes for data being
    // passed to the thread (see https://docs.rs/crossbeam/0.8.2/crossbeam/thread/struct.Scope.html)
    extern crate crossbeam;
    use crossbeam::thread;
    use radix_engine::vm::VmModules;

    // this test was inspired by radix_engine "Transfer" benchmark
    #[test]
    fn test_multithread_transfer() {
        // Set up environment.
        let vm_modules = VmModules::default();
        let mut substate_db = InMemorySubstateDatabase::standard();
        ProtocolBuilder::for_simulator()
            .from_bootstrap_to_latest()
            .commit_each_protocol_update(&mut substate_db);

        // Create a key pair
        let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let public_key = private_key.public_key();

        // Create two accounts
        let accounts = (0..2)
            .map(|i| {
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .new_account_advanced(
                        OwnerRole::Fixed(rule!(require(signature(&public_key)))),
                        None,
                    )
                    .build();
                let account = execute_and_commit_transaction(
                    &mut substate_db,
                    &vm_modules,
                    &ExecutionConfig::for_test_transaction(),
                    TestTransaction::new_v1(
                        manifest,
                        hash(format!("Account creation: {i}")),
                        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
                    )
                    .into_executable_unwrap(),
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
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account1, None)
            .build();
        for nonce in 0..10 {
            execute_and_commit_transaction(
                &mut substate_db,
                &vm_modules,
                &ExecutionConfig::for_test_transaction(),
                TestTransaction::new_v1(
                    manifest.clone(),
                    hash(format!("Fill account: {}", nonce)),
                    btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
                )
                .into_executable_unwrap(),
            )
            .expect_commit(true);
        }

        // Create a transfer manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account1, XRD, dec!("0.000001"))
            .try_deposit_entire_worktop_or_abort(account2, None)
            .build();

        // Spawning threads that will attempt to withdraw some XRD amount from account1 and deposit to
        // account2
        thread::scope(|s| {
            for _i in 0..20 {
                // Note - we run the same transaction on all threads, but don't commit anything
                s.spawn(|_| {
                    let receipt = execute_transaction(
                        &substate_db,
                        &vm_modules,
                        &ExecutionConfig::for_test_transaction(),
                        TestTransaction::new_v1(
                            manifest.clone(),
                            hash(format!("Transfer")),
                            btreeset![NonFungibleGlobalId::from_public_key(&public_key,)],
                        )
                        .into_executable_unwrap(),
                    );
                    receipt.expect_commit_success();
                    println!("receipt = {:?}", receipt);
                });
            }
        })
        .unwrap();
    }
}
