use radix_engine::system::bootstrap::Bootstrapper;
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

#[test]
fn system_database_checker_should_report_missing_owner_error_on_broken_db() {
    // Arrange
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    let mut bootstrapper = Bootstrapper::new(&mut substate_db, vm, true);
    bootstrapper.bootstrap_test_default().unwrap();
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

    // Act
    let checker = SystemDatabaseChecker::new();
    let checker_result = checker.check_db(&substate_db);

    // Assert
    let error = checker_result.expect_err("Should be an error");
    assert_eq!(
        error,
        SystemDatabaseCheckError::NodeError(SystemNodeCheckError::MissingExpectedFields)
    );
}
