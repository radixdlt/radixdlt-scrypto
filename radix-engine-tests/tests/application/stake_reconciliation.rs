use core::fmt::Write;
use radix_common::prelude::*;
use radix_engine::updates::BabylonSettings;
use radix_substate_store_interface::db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper};
use scrypto_test::prelude::*;

// HELP DEBUGGING:
// If this test diverges, it's often because of a change to genesis, which has made a substate
// bigger and changed the costing.
//
// The best way to debug this is to run the following to dump a new trace.
// cargo test --package radix-engine-tests -- application::stake_reconciliation::test_stake_reconciliation --exact --show-output > radix-engine-tests/tests/application/reconciliation_log.txt
//
// And look for line/s which are different. Likely it'll be some substate write.
// Then look further down this test for a block of code to uncomment, which can be used to read that
// value from the database.
#[test]
fn test_stake_reconciliation() {
    // Arrange
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_kernel_trace()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| BabylonSettings::test_minimal())
                .only_babylon()
        })
        .build();
    let (account_pk, _, account) = ledger.new_account(false);

    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .register_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );
    receipt.expect_commit_success();

    // Store current DB substate value hashes for comparision after staking execution
    let mut pre_transaction_substates: IndexMap<(DbPartitionKey, DbSortKey), Vec<u8>> = indexmap!();
    let db = ledger.substate_db();
    let old_keys: Vec<DbPartitionKey> = db.list_partition_keys().collect();
    for key in old_keys {
        let entries = db.list_raw_values_from_db_key(&key, None);
        for (sort_key, value) in entries {
            pre_transaction_substates.insert((key.clone(), sort_key), value);
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
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );

    let address_encoder = AddressBech32Encoder::for_simulator();
    let receipt_display_context = TransactionReceiptDisplayContextBuilder::new()
        .encoder(&address_encoder)
        .display_state_updates(true)
        .use_ansi_colors(false)
        .schema_lookup_from_db(ledger.substate_db())
        .set_max_substate_length_to_display(1000000)
        .build();

    // UNCOMMENT THESE LINES TO PRINT OUT A NICE DISPLAY OF A SINGLE SUBSTATE
    // {
    //     let mut error_message = String::new();
    //     let node_id = NodeId(hex::decode("0d906318c6318c6c4e1b40cc6318c6318cf7bfd5d45f48c686318c6318c6").unwrap().try_into().unwrap());
    //     let partition_num = PartitionNumber(1);
    //     let substate_key = SubstateKey::Map(ScryptoRawPayload::from_valid_payload(vec![92, 32, 7, 32, 220, 0, 156, 5, 6, 83, 96, 189, 222, 100, 29, 145, 160, 147, 193, 127, 71, 54, 135, 62, 103, 35, 126, 168, 230, 117, 203, 71, 36, 132, 155, 157]));
    //     let value_from_database: ScryptoRawValue = ledger.substate_db()
    //         .get_raw_substate(&node_id, partition_num, substate_key)
    //         .unwrap();
    //     let substate_value = value_from_database.into_payload_bytes();
    //     let state_updates = StateUpdates {
    //         by_node: indexmap! {
    //             node_id => NodeStateUpdates::Delta {
    //                 by_partition: indexmap! {
    //                     partition_num => PartitionStateUpdates::Delta {
    //                         by_substate: indexmap! {
    //                             substate_key.clone() => DatabaseUpdate::Set(substate_value.clone())
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     };
    //     let system_structure = SystemStructure::resolve(ledger.substate_db(), &state_updates, &vec![]);
    //     format_substate_value(
    //         &mut error_message,
    //         &system_structure.substate_system_structures[&node_id][&partition_num][&substate_key],
    //         &receipt_display_context,
    //         &substate_value,
    //     ).unwrap();
    //     panic!(error_message)
    // }

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

    let commit_result = receipt.expect_commit(true).clone();
    let events = commit_result.application_events;
    for (idx, event) in events.iter().enumerate() {
        let name = ledger.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
        assert_eq!(name, expected_events[idx]);
    }

    println!("{:-^120}", "Application DB Partitions");

    let db = ledger.substate_db();
    let post_transaction_partitions = db.list_partition_keys();

    let mut new_substates_count = 0;
    let mut changed_substates_count = 0;
    let mut same_substates_count = 0;

    let expected_updated_substates = hashmap! {
        (
            // internal_vault_sim1tz9uaalv8g3ahmwep2trlyj2m3zn7rstm9pwessa3k56me2fcduq2u
            DbPartitionKey {
                node_key: unhex("06ef5035dba9d29588fa280b760358845b5070f1588bcef7ec3a23dbedd90a963f924adc453f0e0bd942ecc21d8da9ade549"),
                partition_num: 64,
            },
            DbSortKey(unhex("00"))
        ) => (
            unhex("5c2200012102220001a0005eb9725df575d24549c772614213000000000000000000220000"), // OLD
            unhex("5c2200012102220001a0805a4d3774b32eca4549c772614213000000000000000000220000"), // NEW
        ),
        (
            // validator_sim1s0u4eunqps02ap3t3drdplqhj4uaadxyhal8tue7aqeyk2qnxe3sjf
            DbPartitionKey {
                node_key: unhex("b5306d54d9fec7bf100aa1d246435420585bf18e83f95cf2600c1eae862b8b46d0fc179579deb4c4bf7e75f33ee8324b2813"),
                partition_num: 64,
            },
            DbSortKey(unhex("00"))
        ) => (
            unhex("5c2200012102220001210e2200002007210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179801010100a0000064a7b3b6e00d00000000000000000000000000000000220000805d38dff7505ea2f90369d5b0cbaaa89528a3091332fc7c16b92b3bc9472a905855aa6593187a4797ff138d1c0f22b26e1c3df3e90fda88e163afaf4072809a8471340c5eced86602d6071b7c90f2d28afef5ae2007a95e53621dbd099058b2831bd25d5dd2c01c5d8a13ebd9ce35c274a8f95452b5b17144c45716905879ad39d02fb58f3a9a6b6c002582b3279a6e962d641d1e3ccbc438608a90583cf49431a942b361ceb87c00e0a3fcc2431e011d7df83bac744c4d4916230aa000a0000000000000000000000000000000000000000000000000220000"), // OLD
            unhex("5c2200012102220001210e2201012102200702ffff2007205c8083f95cf2600c1eae862b8b46d0fc179579deb4c4bf7e75f33ee8324b28132007210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179801010100a0000064a7b3b6e00d00000000000000000000000000000000220000805d38dff7505ea2f90369d5b0cbaaa89528a3091332fc7c16b92b3bc9472a905855aa6593187a4797ff138d1c0f22b26e1c3df3e90fda88e163afaf4072809a8471340c5eced86602d6071b7c90f2d28afef5ae2007a95e53621dbd099058b2831bd25d5dd2c01c5d8a13ebd9ce35c274a8f95452b5b17144c45716905879ad39d02fb58f3a9a6b6c002582b3279a6e962d641d1e3ccbc438608a90583cf49431a942b361ceb87c00e0a3fcc2431e011d7df83bac744c4d4916230aa000a0000000000000000000000000000000000000000000000000220000"), // NEW
        ),
        (
            // consensusmanager_sim1scxxxxxxxxxxcnsmgrxxxxxxxxx000999665565xxxxxxxxxxc06cl
            DbPartitionKey {
                node_key: unhex("14a7d055604bf45858649fde5f5ff598e6f99e0e860c6318c6318c6c4e1b40cc6318c6318cf7bca52eb54a6a86318c6318c6"),
                partition_num: 64,
            },
            DbSortKey(unhex("02"))
        ) => (
            unhex("5c220001210222000121022307a0010080a8314ac5015409000000000000000000000000000000009058619833de031de3aad69cad02a22656e083e307fb617b28e1b275bd7ed7220000"), // OLD
            unhex("5c220001210222000121022307a0010060a90c993fd2650b000000000000000000000000000000009058619833de031de3aad69cad02a22656e083e307fb617b28e1b275bd7ed7220000"), // NEW
        ),
        (
            // internal_vault_sim1tp265evnrpay09llzwx3crezkfhpc00nay8a4z8pvwh67srj7vchdx
            DbPartitionKey {
                node_key: unhex("227a239cf90d2529e6fea9dbd78f356f4353465f5855aa6593187a4797ff138d1c0f22b26e1c3df3e90fda88e163afaf4072"),
                partition_num: 64,
            },
            DbSortKey(unhex("00"))
        ) => (
            unhex("5c2200012102220001a0000000000000000000000000000000000000000000000000220000"), // OLD
            unhex("5c2200012102220001a00000e8890423c78a00000000000000000000000000000000220000"), // NEW
        ),
        (
            // internal_vault_sim1trfekxxzevygt2uwrknmykuh8m2538myupm9d954d9q658844cxfp8
            DbPartitionKey {
                node_key: unhex("f0353d769d1ca066ac3724959a79786fb720237a58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5"),
                partition_num: 64,
            },
            DbSortKey(unhex("00"))
        ) => (
            unhex("5c2200012102220001a0985575f1801c1cdae1030000000000000000000000000000220000"), // OLD
            unhex("5c2200012102220001a098558d677cf9544fe1030000000000000000000000000000220000"), // NEW
        ),
        (
            // resource_sim1t5udla6st630jqmf6kcvh24gj552xzgnxt78c94e9vauj3e27ued99
            DbPartitionKey {
                node_key: unhex("482087995e0e15a84caf3b614aa9c1406dacea455d38dff7505ea2f90369d5b0cbaaa89528a3091332fc7c16b92b3bc9472a"),
                partition_num: 64,
            },
            DbSortKey(unhex("01"))
        ) => (
            unhex("5c2200012102220001a0000000000000000000000000000000000000000000000000220000"), // OLD
            unhex("5c2200012102220001a00000e8890423c78a00000000000000000000000000000000220000"), // NEW
        ),
        (
            // internal_vault_sim1tpsesv77qvw782kknjks9g3x2msg8cc8ldshk28pkf6m6lkhun3sel
            DbPartitionKey {
                node_key: unhex("f3052b1133393854e7f8ddc613929df4d35c775858619833de031de3aad69cad02a22656e083e307fb617b28e1b275bd7ed7"),
                partition_num: 64,
            },
            DbSortKey(unhex("00"))
        ) => (
            unhex("5c2200012102220001a0005163948a03a81200000000000000000000000000000000220000"), // OLD
            unhex("5c2200012102220001a0c05219327fa4cb1600000000000000000000000000000000220000"), // NEW
        ),
    };

    let expected_new_substates = hashmap! {
        (
            // consensusmanager_sim1scxxxxxxxxxxcnsmgrxxxxxxxxx000999665565xxxxxxxxxxc06cl
            DbPartitionKey {
                node_key: unhex("14a7d055604bf45858649fde5f5ff598e6f99e0e860c6318c6318c6c4e1b40cc6318c6318cf7bca52eb54a6a86318c6318c6"),
                partition_num: 65,
            },
            DbSortKey(unhex("ffff745eed445b4272ae286fc448bf254eb58d46b9805c8083f95cf2600c1eae862b8b46d0fc179579deb4c4bf7e75f33ee8324b2813"))
        ) => unhex("5c22000122000121022007210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798a00000e8890423c78a00000000000000000000000000000000"),
        (
            // account_sim1c8m6h4yv2x9ca0wx5ddtl0nctqmjt2t740wfjgj9w8sdz82zf8ppcr
            DbPartitionKey {
                node_key: unhex("9c9f27834134d3791419f503a9ce37e78be559a7c1f7abd48c518b8ebdc6a35abfbe78583725a97eabdc99224571e0d11d42"),
                partition_num: 65,
            },
            DbSortKey(unhex("c2da5b263e3091554fd575a7357805a17c900da85c805d38dff7505ea2f90369d5b0cbaaa89528a3091332fc7c16b92b3bc9472a"))
        ) => unhex("5c220001210222010122000190582326a6d3d5955d57f0c47788a2eb931113b3bb5b82e2f73b592c8ab770220000"),
        (
            // internal_vault_sim1tq3jdfkn6k2464lsc3mc3ghtjvg38vamtwpw9aemtykg4dmsptjpm5
            DbPartitionKey {
                node_key: unhex("6f1e8cf4efda20aea8acd7b07c0084d5ff316732582326a6d3d5955d57f0c47788a2eb931113b3bb5b82e2f73b592c8ab770"),
                partition_num: 0,
            },
            DbSortKey(unhex("00"))
        ) => unhex("5c220001210221052102800d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c60c0d46756e6769626c655661756c742103090100000009000000000900000000220001805d38dff7505ea2f90369d5b0cbaaa89528a3091332fc7c16b92b3bc9472a200c00202200220100"),
        (
            // internal_vault_sim1tq3jdfkn6k2464lsc3mc3ghtjvg38vamtwpw9aemtykg4dmsptjpm5
            DbPartitionKey {
                node_key: unhex("6f1e8cf4efda20aea8acd7b07c0084d5ff316732582326a6d3d5955d57f0c47788a2eb931113b3bb5b82e2f73b592c8ab770"),
                partition_num: 64,
            },
            DbSortKey(unhex("00"))
        ) => unhex("5c2200012102220001a00000e8890423c78a00000000000000000000000000000000220000"),
    };

    let post_transaction_partitions: Vec<_> = post_transaction_partitions.collect();

    let mut error_message = String::new();

    for (full_key, (expected_old_value, _)) in expected_updated_substates.iter() {
        let database_value = &pre_transaction_substates[full_key];
        let (node_id, partition) = SpreadPrefixKeyMapper::from_db_partition_key(&full_key.0);
        // Luckily they're all fields
        let substate_key = SpreadPrefixKeyMapper::from_db_sort_key::<FieldKey>(&full_key.1);
        let address = AddressBech32Encoder::for_simulator()
            .encode(&node_id.0)
            .unwrap();
        if database_value != expected_old_value {
            let substate_structure = &commit_result.system_structure.substate_system_structures
                [&node_id][&partition][&substate_key];
            write!(&mut error_message, "\nThe pre-transaction value of updated substate under {address} {partition:?} {substate_key:?} has changed.").unwrap();
            write!(&mut error_message, "\n\nEXPECTED:").unwrap();
            format_receipt_substate_value(
                &mut error_message,
                &substate_structure,
                &receipt_display_context,
                &expected_old_value,
            )
            .unwrap();
            write!(&mut error_message, "\nACTUAL:").unwrap();
            format_receipt_substate_value(
                &mut error_message,
                &substate_structure,
                &receipt_display_context,
                &database_value,
            )
            .unwrap();
            write!(&mut error_message, "\n").unwrap();
        }
        // For printing:
        // let (db_partition_key, db_sort_key) = full_key;
        // println!(
        //     "            (
        //         // {}
        //         DbPartitionKey {{
        //             node_key: unhex({:?}),
        //             partition_num: {:?},
        //         }},
        //         DbSortKey(unhex({:?}))
        //     ) => (
        //         unhex({:?}), // OLD
        //         unhex({:?}), // NEW
        //     ),",
        //     address,
        //     hex::encode(&db_partition_key.node_key),
        //     db_partition_key.partition_num,
        //     hex::encode(&db_sort_key.0),
        //     hex::encode(database_value),
        //     hex::encode(new_value)
        // );
    }
    if error_message.len() > 0 {
        panic!("{}", error_message);
    }

    for key in post_transaction_partitions {
        let partition_entries = ledger.substate_db().list_raw_values_from_db_key(&key, None);
        for (sort_key, current_value) in partition_entries {
            let full_key = (key.clone(), sort_key.clone());
            let address = AddressBech32Encoder::for_simulator()
                .encode(
                    &SpreadPrefixKeyMapper::from_db_partition_key(&full_key.0)
                        .0
                         .0,
                )
                .unwrap();

            if let Some(old_value) = pre_transaction_substates.get(&full_key) {
                if old_value == &current_value {
                    same_substates_count += 1;
                } else {
                    changed_substates_count += 1;
                    let expected_updated_value =
                        expected_updated_substates.get(&full_key).map(|x| &x.1);
                    assert_eq!(
                        Some(&current_value),
                        expected_updated_value,
                        "The resultant value of updated substate under {} is not expected: {:?}",
                        address,
                        full_key
                    );
                }
            } else {
                new_substates_count += 1;
                assert_eq!(
                    Some(&current_value),
                    expected_new_substates.get(&full_key),
                    "The resultant value of new substate under {} is not expected: {:?}",
                    address,
                    full_key
                );
            }
        }
    }

    println!("{:-^120}", "Report end");

    assert_eq!(new_substates_count, expected_new_substates.len());
    assert_eq!(changed_substates_count, expected_updated_substates.len());
    assert_eq!(same_substates_count, 578);
}

fn unhex(input: &'static str) -> Vec<u8> {
    hex::decode(input).unwrap()
}
