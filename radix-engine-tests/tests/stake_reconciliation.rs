use radix_engine_common::prelude::*;
use radix_engine_store_interface::interface::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_stake_reconciliation() {
    // Arrange
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let mut test_runner = TestRunnerBuilder::new().build();
    let (account_pk, _, account) = test_runner.new_account(false);

    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );
    receipt.expect_commit_success();

    // Store current DB substate values hases for comparision after staking execution
    let mut old_values_map: HashMap<(DbPartitionKey, DbSortKey), Hash> = HashMap::new();
    let db = test_runner.substate_db();
    let old_keys: Vec<DbPartitionKey> = db.list_partition_keys().collect();
    for key in old_keys {
        let entries = db.list_entries(&key);
        for (sort_key, value) in entries {
            old_values_map.insert((key.clone(), sort_key), hash(value));
        }
    }

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .withdraw_from_account(account, XRD, 10)
        .take_all_from_worktop(XRD, "stake")
        .stake_validator_as_owner(validator_address, "stake")
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    // Assert
    println!("{:-^120}", "Application Events");

    // ordered list of expected events
    let expected_events = vec![
        "LockFeeEvent",
        "WithdrawEvent",
        "WithdrawEvent",
        "MintFungibleResourceEvent",
        "DepositEvent",
        "StakeEvent",
        "VaultCreationEvent",
        "DepositEvent",
        "DepositEvent",
        "PayFeeEvent",
        "DepositEvent",
        "BurnFungibleResourceEvent",
    ];

    let events = receipt.expect_commit(true).clone().application_events;
    for (idx, event) in events.iter().enumerate() {
        let name = test_runner.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
        assert_eq!(name, expected_events[idx]);
    }

    println!("{:-^120}", "Application DB Partitions");

    let db = test_runner.substate_db();
    let keys = db.list_partition_keys();

    let mut new_substates_count = 0;
    let mut changed_substates_count = 0;
    let mut same_substates_count = 0;

    let expected_changed_substates: HashMap<(usize, usize), Hash> = HashMap::from([
        (
            (1, 0),
            Hash::from_str("dde1676c6ba8cb78fb6b6be0727c355df7c0076d3a22e0a3da5b03d520c57e66")
                .unwrap(),
        ),
        (
            (44, 2),
            Hash::from_str("e74d0fc250851cb2b81810507b529e2153fca074b7825994fe94cdc81f80dd3d")
                .unwrap(),
        ),
        (
            (58, 0),
            Hash::from_str("f9be16af54ca0b0ffa27cc4491c2b13819bc990d8f282e48817af73bf9d9aa13")
                .unwrap(),
        ),
        (
            (117, 1),
            Hash::from_str("f9be16af54ca0b0ffa27cc4491c2b13819bc990d8f282e48817af73bf9d9aa13")
                .unwrap(),
        ),
        (
            (233, 0),
            Hash::from_str("147feb57af0299f21d85b496c99c8292676c51307f2a45713355f4bc14ef7c8b")
                .unwrap(),
        ),
        (
            (263, 0),
            Hash::from_str("f96ad299fcfb0c919c9faa48d342318e6ddf7da6db1d6a489b51442affdff33d")
                .unwrap(),
        ),
        (
            (265, 0),
            Hash::from_str("f26dfee9e3f5c299bcb2a8acbd5a12421cb2dd2b2bc1b62fc6fed6a7ce82487e")
                .unwrap(),
        ),
    ]);

    let expected_new_substates: HashMap<(usize, usize), Hash> = HashMap::from([
        (
            (45, 0),
            Hash::from_str("997ed5b983cc38b9984e6ab1d58ea7cbd291f6c033f73fc7aae69b01647228f3")
                .unwrap(),
        ),
        (
            (151, 0),
            Hash::from_str("b086f3e0039193e36f6d9bdd1ad6ee4e0f8f487521167fda853fab7f553ebfb4")
                .unwrap(),
        ),
        (
            (152, 0),
            Hash::from_str("f9be16af54ca0b0ffa27cc4491c2b13819bc990d8f282e48817af73bf9d9aa13")
                .unwrap(),
        ),
        (
            (214, 1),
            Hash::from_str("02e97ec6d458301ed5dce097bb650c97a6a22b653600183c828ec58ddd476072")
                .unwrap(),
        ),
    ]);

    for (idx, key) in keys.enumerate() {
        let partition_entries = test_runner.substate_db().list_entries(&key);
        for (sidx, (sort_key, value)) in partition_entries.enumerate() {
            let value_hash = hash(value);

            if let Some(old_value_hash) = old_values_map.get(&(key.clone(), sort_key)) {
                if old_value_hash == &value_hash {
                    same_substates_count += 1;
                } else {
                    changed_substates_count += 1;
                    println!("Partition({}) Substate({}) changed", idx, sidx);
                    assert_eq!(
                        &value_hash,
                        expected_changed_substates.get(&(idx, sidx)).unwrap()
                    );
                }
            } else {
                new_substates_count += 1;
                println!("Partition({}) Substate({}) new", idx, sidx);
                assert_eq!(
                    &value_hash,
                    expected_new_substates.get(&(idx, sidx)).unwrap()
                );
            }
        }
    }

    println!("{:-^120}", "Report end");

    assert_eq!(new_substates_count, 4);
    assert_eq!(changed_substates_count, 7);
    assert_eq!(same_substates_count, 578);
}
