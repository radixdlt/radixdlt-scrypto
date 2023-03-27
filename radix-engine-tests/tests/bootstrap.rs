
use super::*;
use crate::{ledger::InMemorySubstateStore, wasm::DefaultWasmEngine};
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let substate_store = InMemorySubstateStore::new();
    let mut initial_validator_set = BTreeMap::new();
    let public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let account_address = ComponentAddress::virtual_account_from_public_key(&public_key);
    initial_validator_set.insert(
        EcdsaSecp256k1PublicKey([0; 33]),
        (Decimal::one(), account_address),
    );
    let genesis_transaction =
        create_genesis(initial_validator_set, BTreeMap::new(), 1u64, 1u64, 1u64);

    let transaction_receipt = execute_transaction(
        &substate_store,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::genesis().with_trace(true),
        &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
    );
    #[cfg(not(feature = "alloc"))]
    println!("{:?}", transaction_receipt);

    transaction_receipt
        .expect_commit(true)
        .next_epoch()
        .expect("There should be a new epoch.");

    assert!(transaction_receipt
        .expect_commit(true)
        .new_package_addresses()
        .contains(&PACKAGE_PACKAGE));
    let genesis_receipt = genesis_result(&transaction_receipt);
    assert_eq!(genesis_receipt.faucet_component, FAUCET_COMPONENT);
}

#[test]
fn test_genesis_xrd_allocation_to_accounts() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let mut substate_store = InMemorySubstateStore::new();
    let account_public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let account_component_address = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(account_public_key.clone()),
    );
    let allocation_amount = dec!("100");
    let mut account_xrd_allocations = BTreeMap::new();
    account_xrd_allocations.insert(account_public_key, allocation_amount);
    let genesis_transaction =
        create_genesis(BTreeMap::new(), account_xrd_allocations, 1u64, 1u64, 1u64);

    let transaction_receipt = execute_transaction(
        &substate_store,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::genesis(),
        &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
    );

    let commit_result = transaction_receipt.expect_commit(true);
    commit_result.state_updates.commit(&mut substate_store);

    assert!(transaction_receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc| rc.amount == allocation_amount
            && rc.node_id == RENodeId::GlobalObject(account_component_address.into())));
}

#[test]
fn test_encode_and_decode_validator_init() {
    let t = ManifestValidatorInit {
        validator_account_address: ComponentAddress::AccessController([0u8; 26]),
        initial_stake: ManifestBucket(1),
        stake_account_address: ComponentAddress::AccessController([0u8; 26]),
    };

    let bytes = manifest_encode(&t).unwrap();
    let decoded: ManifestValidatorInit = manifest_decode(&bytes).unwrap();
    assert_eq!(decoded, t);
}
