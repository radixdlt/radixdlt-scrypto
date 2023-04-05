use radix_engine::kernel::interpreters::*;
use radix_engine::system::bootstrap::{bootstrap_with_validator_set, create_genesis};
use radix_engine::transaction::{execute_transaction, ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::epoch_manager::ManifestValidatorInit;
use radix_engine_stores::interface::CommittableSubstateDatabase;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();

    let transaction_receipt = bootstrap_with_validator_set(
        &mut substate_db,
        &scrypto_interpreter,
        BTreeMap::new(),
        BTreeMap::new(),
        1u64,
        1u64,
        1u64,
        true,
    )
    .unwrap();
    transaction_receipt.expect_commit_success();
}

#[test]
fn test_genesis_xrd_allocation_to_accounts() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let account_public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let account_address = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(account_public_key.clone()),
    );
    let allocation_amount = dec!("100");
    let mut account_xrd_allocations = BTreeMap::new();
    account_xrd_allocations.insert(account_public_key, allocation_amount);
    let genesis_transaction =
        create_genesis(BTreeMap::new(), account_xrd_allocations, 1u64, 1u64, 1u64);

    let transaction_receipt = execute_transaction(
        &substate_db,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::genesis(),
        &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
    );

    let commit_result = transaction_receipt.expect_commit(true);
    substate_db.commit(&commit_result.state_updates).unwrap();

    assert!(transaction_receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc| rc.amount == allocation_amount && rc.node_id == account_address.into()));
}

#[test]
fn test_encode_and_decode_validator_init() {
    let t = ManifestValidatorInit {
        validator_account_address: component_address(EntityType::GlobalAccessController, 1),
        initial_stake: ManifestBucket(1),
        stake_account_address: component_address(EntityType::GlobalAccessController, 1),
    };

    let bytes = manifest_encode(&t).unwrap();
    let decoded: ManifestValidatorInit = manifest_decode(&bytes).unwrap();
    assert_eq!(decoded, t);
}
