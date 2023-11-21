use radix_engine_common::prelude::*;
use transaction::prelude::*;
use scrypto_unit::*;
use radix_engine_store_interface::interface::*;

#[test]
fn test_stake_reconcilation() {
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

    let events = receipt.expect_commit(true).clone().application_events;
    for event in &events {
        let name = test_runner.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
    }

    println!("{:-^120}", "Application DB Partitions");

    let db = test_runner.substate_db();
    let keys = db.list_partition_keys();

    let mut new_substates_count = 0;
    let mut changed_substates_count = 0;
    let mut same_substates_count = 0;

    for (_idx, key) in keys.enumerate() {
        let partition_entries = test_runner.substate_db().list_entries(&key);
        for (_sidx, (sort_key, value)) in partition_entries.enumerate() {
            if let Some(value_hash) = old_values_map.get(&(key.clone(), sort_key)) {
                if value_hash == &hash(value) {
                    same_substates_count += 1;
                } else {
                    changed_substates_count += 1;
                }
            } else {
                new_substates_count += 1;
            }
        }
    }

    assert_eq!(new_substates_count, 4);
    assert_eq!(changed_substates_count, 7);
    assert_eq!(same_substates_count, 578);
}

