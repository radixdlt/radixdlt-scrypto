use radix_engine::engine::{ApplicationError, RuntimeError};
use radix_engine::model::{AccessControllerError, AuthZoneError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use scrypto_unit::TestRunner;
use transaction::{builder::ManifestBuilder, model::TransactionManifest};

#[test]
pub fn creating_an_access_controller_succeeds() {
    AccessControllerTestRunner::new(10);
}

#[test]
pub fn role_cant_quick_confirm_a_ruleset_it_proposed() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );

    // Act
    let receipt = test_runner.quick_confirm_recovery(
        Role::Recovery,
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );

    // Assert
    receipt.expect_specific_failure(is_no_valid_proposed_rule_set_exists_error)
}

#[test]
pub fn quick_confirm_non_existent_recovery_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );

    // Act
    let receipt = test_runner.quick_confirm_recovery(
        Role::Recovery,
        Role::Recovery,
        rule!(require(PACKAGE_TOKEN)),
        rule!(require(PACKAGE_TOKEN)),
        rule!(require(PACKAGE_TOKEN)),
    );

    // Assert
    receipt.expect_specific_failure(is_no_valid_proposed_rule_set_exists_error)
}

#[test]
pub fn initiating_recovery_multiple_times_as_the_same_role_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
                AccessControllerError::RecoveryForThisRoleAlreadyExists {
                    role: Role::Recovery
                }
            ))
        )
    })
}

#[test]
pub fn timed_confirm_recovery_before_delay_passes_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner.initiate_recovery(
        Role::Primary,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );
    test_runner.push_time_forward(9);

    // Act
    let receipt = test_runner.timed_confirm_recovery(
        Role::Primary,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );

    // Assert
    receipt.expect_specific_failure(is_timed_recovery_delay_has_not_elapsed_error);
}

#[test]
pub fn timed_confirm_recovery_after_delay_passes_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner.initiate_recovery(
        Role::Primary,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );
    test_runner.push_time_forward(10);

    // Act
    let receipt = test_runner.timed_confirm_recovery(
        Role::Primary,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
    );

    // Assert
    receipt.expect_commit_success();
}

//=============
// State Tests
//=============

mod normal_operations_with_primary_unlocked {
    use super::*;

    const TIMED_RECOVERY_DELAY_IN_HOURS: u16 = 10;

    fn setup_environment() -> AccessControllerTestRunner {
        AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_HOURS)
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.create_proof(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn update_timed_recovery_delay() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.update_timed_recovery_delay(role, 100);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn lock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.lock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn unlock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, None),
            (Role::Confirmation, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.unlock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn quick_confirm_recovery() {
        // Arrange
        let test_vectors = [
            (
                Role::Primary,  // As role
                Role::Recovery, // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery, // As role
                Role::Primary,  // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
        ];

        for (role, proposer, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.quick_confirm_recovery(
                role,
                proposer,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn timed_confirm_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn cancel_recovery_attempt() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }
}

mod normal_operations_with_primary_locked {
    use super::*;

    const TIMED_RECOVERY_DELAY_IN_HOURS: u16 = 10;

    fn setup_environment() -> AccessControllerTestRunner {
        let mut test_runner = AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_HOURS);
        test_runner
            .lock_primary_role(Role::Primary)
            .expect_commit_success();
        test_runner
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.create_proof(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn update_timed_recovery_delay() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.update_timed_recovery_delay(role, 100);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn lock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.lock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn unlock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, None),
            (Role::Confirmation, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.unlock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn quick_confirm_recovery() {
        // Arrange
        let test_vectors = [
            (
                Role::Primary,  // As role
                Role::Recovery, // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery, // As role
                Role::Primary,  // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
        ];

        for (role, proposer, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.quick_confirm_recovery(
                role,
                proposer,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn timed_confirm_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn cancel_recovery_attempt() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }
}

mod recovery_mode_with_primary_unlocked {
    use super::*;

    const TIMED_RECOVERY_DELAY_IN_HOURS: u16 = 10;

    fn setup_environment() -> AccessControllerTestRunner {
        let mut test_runner = AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_HOURS);
        test_runner
            .initiate_recovery(
                Role::Recovery,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            )
            .expect_commit_success();
        test_runner
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.create_proof(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn update_timed_recovery_delay() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.update_timed_recovery_delay(role, 100);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, None),
            (
                Role::Recovery,
                Some(is_recovery_for_this_role_already_exists_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn lock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.lock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn unlock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, None),
            (Role::Confirmation, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.unlock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn quick_confirm_recovery() {
        // Arrange
        let test_vectors = [
            (
                Role::Primary,  // As role
                Role::Recovery, // Proposer
                None,
            ),
            (
                Role::Recovery, // As role
                Role::Primary,  // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
        ];

        for (role, proposer, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.quick_confirm_recovery(
                role,
                proposer,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn timed_confirm_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery,
                Some(is_timed_recovery_delay_has_not_elapsed_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn cancel_recovery_attempt() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }
}

mod recovery_mode_with_primary_locked {
    use super::*;

    const TIMED_RECOVERY_DELAY_IN_HOURS: u16 = 10;

    fn setup_environment() -> AccessControllerTestRunner {
        let mut test_runner = AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_HOURS);
        test_runner
            .lock_primary_role(Role::Primary)
            .expect_commit_success();
        test_runner
            .initiate_recovery(
                Role::Recovery,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            )
            .expect_commit_success();
        test_runner
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.create_proof(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn update_timed_recovery_delay() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, Some(is_auth_assertion_error)),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.update_timed_recovery_delay(role, 100);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (
                Role::Recovery,
                Some(is_recovery_for_this_role_already_exists_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn lock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.lock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn unlock_primary_role() {
        // Arrange
        let test_vectors = [
            (Role::Primary, Some(is_auth_assertion_error)),
            (Role::Recovery, None),
            (Role::Confirmation, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.unlock_primary_role(role);

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn quick_confirm_recovery() {
        // Arrange
        let test_vectors = [
            (
                Role::Primary,  // As role
                Role::Recovery, // Proposer
                None,
            ),
            (
                Role::Recovery, // As role
                Role::Primary,  // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
        ];

        for (role, proposer, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.quick_confirm_recovery(
                role,
                proposer,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn timed_confirm_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (
                Role::Recovery,
                Some(is_timed_recovery_delay_has_not_elapsed_error),
            ),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }

    #[test]
    pub fn cancel_recovery_attempt() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_no_valid_proposed_rule_set_exists_error),
            ),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_assertion_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
            );

            // Assert
            match error_assertion_function {
                None => {
                    receipt.expect_commit_success();
                }
                Some(function) => receipt.expect_specific_failure(function),
            };
        }
    }
}

//==================
// Helper Functions
//==================

type ErrorCheckFunction = fn(&RuntimeError) -> bool;

fn is_auth_assertion_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(
            AuthZoneError::AssertAccessRuleError(..)
        ))
    )
}

fn is_no_valid_proposed_rule_set_exists_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::NoValidProposedRuleSetExists
        ))
    )
}

fn is_recovery_for_this_role_already_exists_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::RecoveryForThisRoleAlreadyExists { .. }
        ))
    )
}

fn is_timed_recovery_delay_has_not_elapsed_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::TimedRecoveryDelayHasNotElapsed { .. }
        ))
    )
}

#[allow(dead_code)]
struct AccessControllerTestRunner {
    pub test_runner: TestRunner,

    pub account: (ComponentAddress, PublicKey),

    pub access_controller_component_address: ComponentAddress,
    pub primary_role_badge: ResourceAddress,
    pub recovery_role_badge: ResourceAddress,
    pub confirmation_role_badge: ResourceAddress,

    pub timed_recovery_delay_in_hours: u16,
}

#[allow(dead_code)]
impl AccessControllerTestRunner {
    pub fn new(timed_recovery_delay_in_hours: u16) -> Self {
        let mut test_runner = TestRunner::new(false);

        // Creating a new account - this is where the badges will be held
        let (public_key, _, account_component) = test_runner.new_account(false);

        // Creating the resource to be protected
        let controlled_asset = test_runner.create_fungible_resource(1.into(), 0, account_component);

        // Creating three badges for the three roles.
        let primary_role_badge =
            test_runner.create_fungible_resource(1.into(), 0, account_component);
        let recovery_role_badge =
            test_runner.create_fungible_resource(1.into(), 0, account_component);
        let confirmation_role_badge =
            test_runner.create_fungible_resource(1.into(), 0, account_component);

        // Creating the access controller component
        let manifest = ManifestBuilder::new()
            .lock_fee(account_component, 10.into())
            .withdraw_from_account(account_component, controlled_asset)
            .take_from_worktop(controlled_asset, |builder, bucket| {
                builder.create_access_controller(
                    bucket,
                    rule!(require(primary_role_badge)),
                    rule!(require(recovery_role_badge)),
                    rule!(require(confirmation_role_badge)),
                    timed_recovery_delay_in_hours,
                )
            })
            .build();
        let receipt = test_runner.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)].into(),
        );
        receipt.expect_commit_success();

        let access_controller_component_address =
            receipt.new_component_addresses().get(0).unwrap().clone();

        Self {
            test_runner,
            account: (account_component, public_key.into()),

            access_controller_component_address,
            primary_role_badge,
            recovery_role_badge,
            confirmation_role_badge,

            timed_recovery_delay_in_hours,
        }
    }

    pub fn create_proof(&mut self, as_role: Role) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "create_proof",
                scrypto_encode(&AccessControllerCreateProofMethodArgs {}).unwrap(),
            )
            .pop_from_auth_zone(|builder, _| builder)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn update_timed_recovery_delay(
        &mut self,
        as_role: Role,
        timed_recovery_delay_in_hours: u16,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "update_timed_recovery_delay",
                scrypto_encode(&AccessControllerUpdateTimedRecoveryDelayMethodArgs {
                    timed_recovery_delay_in_hours,
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn initiate_recovery(
        &mut self,
        as_role: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "initiate_recovery",
                scrypto_encode(&AccessControllerInitiateRecoveryMethodArgs {
                    role: as_role,
                    rule_set: RuleSet {
                        primary_role: proposed_primary_role,
                        recovery_role: proposed_recovery_role,
                        confirmation_role: proposed_confirmation_role,
                    },
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn quick_confirm_recovery(
        &mut self,
        as_role: Role,
        proposer: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "quick_confirm_recovery",
                scrypto_encode(&AccessControllerQuickConfirmRecoveryMethodArgs {
                    role: as_role,
                    proposer,
                    rule_set: RuleSet {
                        primary_role: proposed_primary_role,
                        recovery_role: proposed_recovery_role,
                        confirmation_role: proposed_confirmation_role,
                    },
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn timed_confirm_recovery(
        &mut self,
        as_role: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "timed_confirm_recovery",
                scrypto_encode(&AccessControllerTimedConfirmRecoveryMethodArgs {
                    role: as_role,
                    rule_set: RuleSet {
                        primary_role: proposed_primary_role,
                        recovery_role: proposed_recovery_role,
                        confirmation_role: proposed_confirmation_role,
                    },
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn cancel_recovery_attempt(
        &mut self,
        as_role: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "cancel_recovery_attempt",
                scrypto_encode(&AccessControllerCancelRecoveryAttemptMethodArgs {
                    role: as_role,
                    rule_set: RuleSet {
                        primary_role: proposed_primary_role,
                        recovery_role: proposed_recovery_role,
                        confirmation_role: proposed_confirmation_role,
                    },
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_primary_role(&mut self, as_role: Role) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "lock_primary_role",
                scrypto_encode(&AccessControllerLockPrimaryRoleMethodArgs {}).unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn unlock_primary_role(&mut self, as_role: Role) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_component_address,
                "unlock_primary_role",
                scrypto_encode(&AccessControllerUnlockPrimaryRoleMethodArgs {}).unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    fn execute_manifest(&mut self, manifest: TransactionManifest) -> TransactionReceipt {
        self.test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&self.account.1)].into(),
        )
    }

    fn manifest_builder(&self, role: Role) -> ManifestBuilder {
        let mut manifest_builder = ManifestBuilder::new();
        let resource_address = match role {
            Role::Primary => self.primary_role_badge,
            Role::Recovery => self.recovery_role_badge,
            Role::Confirmation => self.confirmation_role_badge,
        };
        manifest_builder.create_proof_from_account(self.account.0, resource_address);
        manifest_builder
    }

    fn push_time_forward(&mut self, hours: i64) {
        let current_time = self.test_runner.get_current_time(TimePrecision::Minute);
        let new_time = current_time.add_hours(hours).unwrap();
        self.test_runner
            .set_current_time(new_time.seconds_since_unix_epoch * 1000);
    }
}
