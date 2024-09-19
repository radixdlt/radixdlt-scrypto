use radix_common::prelude::*;
use radix_engine::system::checkers::{
    SystemDatabaseCheckError, SystemDatabaseChecker, SystemNodeCheckError,
};
use radix_engine::updates::ProtocolBuilder;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_interface::interface::*;

#[test]
fn system_database_checker_should_report_missing_owner_error_on_broken_db() {
    // Arrange
    let mut substate_db = InMemorySubstateDatabase::standard();
    ProtocolBuilder::for_simulator()
        .from_bootstrap_to_latest()
        .commit_each_protocol_update(&mut substate_db);

    substate_db.delete_substate(
        PACKAGE_PACKAGE,
        ROLE_ASSIGNMENT_FIELDS_PARTITION,
        SubstateKey::Field(0u8),
    );

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
