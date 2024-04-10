use radix_engine_tests::prelude::*;

#[test]
fn system_database_checker_should_report_missing_owner_error_on_broken_db() {
    // Arrange
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    let mut bootstrapper =
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, true);
    bootstrapper.bootstrap_test_default().unwrap();
    let (node_key, partition_num, sort_key, update) = (
        SpreadPrefixKeyMapper::to_db_node_key(PACKAGE_PACKAGE.as_node_id()),
        SpreadPrefixKeyMapper::to_db_partition_num(
            ROLE_ASSIGNMENT_BASE_PARTITION
                .at_offset(ROLE_ASSIGNMENT_FIELDS_PARTITION_OFFSET)
                .unwrap(),
        ),
        SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Field(0u8)),
        DatabaseUpdate::Delete,
    );
    let remove_owner_update = DatabaseUpdates::from_delta_maps(
        indexmap!(DbPartitionKey {node_key, partition_num} => indexmap!(sort_key => update)),
    );
    substate_db.commit(&remove_owner_update);

    // Act
    let mut checker = SystemDatabaseChecker::<()>::default();
    let checker_result = checker.check_db(&substate_db);

    // Assert
    let error = checker_result.expect_err("Should be an error");
    assert_eq!(
        error,
        SystemDatabaseCheckError::NodeError(SystemNodeCheckError::MissingExpectedFields)
    );
}
