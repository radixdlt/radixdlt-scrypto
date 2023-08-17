use radix_engine::system::bootstrap::{Bootstrapper, GenesisDataChunk, GenesisStakeAllocation};
use radix_engine::system::system_db_checker::{
    SystemDatabaseCheckError, SystemDatabaseChecker, SystemNodeCheckError,
};
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_store_interface::db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper};
use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, PartitionUpdates,
};
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::CustomGenesis;
use transaction::signing::secp256k1::Secp256k1PrivateKey;

#[test]
fn system_database_checker_should_report_missing_owner_error() {
    // Arrange
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let genesis_epoch = Epoch::of(1);
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];
    let mut bootstrapper = Bootstrapper::new(&mut substate_db, vm, true);
    bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            genesis_epoch,
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    // Act
    let remove_owner_update = {
        let mut remove_owner_update = DatabaseUpdates::default();
        let db_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
            PACKAGE_PACKAGE.as_node_id(),
            ROLE_ASSIGNMENT_BASE_PARTITION
                .at_offset(ROLE_ASSIGNMENT_FIELDS_PARTITION_OFFSET)
                .unwrap(),
        );
        let mut partition_updates = PartitionUpdates::default();
        let db_key = SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Field(0u8));
        partition_updates.insert(db_key, DatabaseUpdate::Delete);
        remove_owner_update.insert(db_partition_key, partition_updates);
        remove_owner_update
    };
    substate_db.commit(&remove_owner_update);
    let checker = SystemDatabaseChecker::new();
    let checker_result = checker.check_db(&substate_db);

    // Assert
    let error = checker_result.expect_err("Should be an error");
    assert_eq!(
        error,
        SystemDatabaseCheckError::NodeError(SystemNodeCheckError::MissingExpectedFields)
    );
}
