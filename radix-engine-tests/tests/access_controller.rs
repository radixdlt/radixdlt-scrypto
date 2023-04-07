use radix_engine::blueprints::access_controller::AccessControllerError;
use radix_engine::errors::ApplicationError;
use radix_engine::errors::ModuleError;
use radix_engine::errors::RuntimeError;
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::clock::TimePrecision;
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::TestRunner;
use transaction::{builder::ManifestBuilder, model::TransactionManifest};

#[test]
pub fn creating_an_access_controller_succeeds() {
    AccessControllerTestRunner::new(Some(10));
}

#[test]
pub fn role_cant_quick_confirm_a_ruleset_it_proposed() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(Some(10));
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Act
    let receipt = test_runner.quick_confirm_recovery(
        Role::Recovery,
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error)
}

#[test]
pub fn quick_confirm_non_existent_recovery_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(Some(10));
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Act
    let receipt = test_runner.quick_confirm_recovery(
        Role::Primary,
        Role::Recovery,
        rule!(require(PACKAGE_TOKEN)),
        rule!(require(PACKAGE_TOKEN)),
        rule!(require(PACKAGE_TOKEN)),
        Some(10),
    );

    // Assert
    receipt.expect_specific_failure(is_recovery_proposal_mismatch_error)
}

#[test]
pub fn initiating_recovery_multiple_times_as_the_same_role_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(Some(10));
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Assert
    receipt.expect_specific_failure(is_recovery_already_exists_for_proposer_error)
}

#[test]
pub fn timed_confirm_recovery_before_delay_passes_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(Some(10));
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );
    test_runner.push_time_forward(9);

    // Act
    let receipt = test_runner.timed_confirm_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Assert
    receipt.expect_specific_failure(is_timed_recovery_delay_has_not_elapsed_error);
}

#[test]
pub fn timed_confirm_recovery_after_delay_passes_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(Some(10));
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );
    test_runner.push_time_forward(10);

    // Act
    let receipt = test_runner.timed_confirm_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn timed_confirm_recovery_with_disabled_timed_recovery_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(None);
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );
    test_runner.push_time_forward(10);

    // Act
    let receipt = test_runner.timed_confirm_recovery(
        Role::Recovery,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Assert
    receipt.expect_specific_failure(is_no_timed_recoveries_found_error);
}

#[test]
pub fn timed_confirm_recovery_with_non_recovery_role_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(None);
    test_runner.initiate_recovery(
        Role::Primary,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );
    test_runner.push_time_forward(10);

    // Act
    let receipt = test_runner.timed_confirm_recovery(
        Role::Primary,
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
}

#[test]
pub fn primary_is_unlocked_after_a_successful_recovery() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(Some(10));
    test_runner.initiate_recovery(
        Role::Recovery,
        rule!(require(test_runner.primary_role_badge)),
        rule!(require(RADIX_TOKEN)),
        rule!(require(RADIX_TOKEN)),
        Some(10),
    );
    test_runner
        .lock_primary_role(Role::Recovery)
        .expect_commit_success();

    test_runner.push_time_forward(10);

    test_runner
        .timed_confirm_recovery(
            Role::Recovery,
            rule!(require(test_runner.primary_role_badge)),
            rule!(require(RADIX_TOKEN)),
            rule!(require(RADIX_TOKEN)),
            Some(10),
        )
        .expect_commit_success();

    // Act
    let receipt = test_runner.create_proof(Role::Primary);

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn stop_timed_recovery_with_no_access_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(Some(10));

    let manifest = ManifestBuilder::new()
        .call_method(
            test_runner.access_controller_address,
            "stop_timed_recovery",
            to_manifest_value(&AccessControllerStopTimedRecoveryInput {
                rule_set: RuleSet {
                    primary_role: rule!(require(RADIX_TOKEN)),
                    recovery_role: rule!(require(RADIX_TOKEN)),
                    confirmation_role: rule!(require(RADIX_TOKEN)),
                },
                timed_recovery_delay_in_minutes: Some(10),
            }),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error)
}

#[test]
pub fn quick_confirm_semantics_are_correct() {
    // Arrange
    let test_vectors = [
        (
            Proposer::Primary,
            Role::Primary,
            Some(is_auth_unauthorized_error),
        ),
        (Proposer::Primary, Role::Recovery, None),
        (Proposer::Primary, Role::Confirmation, None),
        (Proposer::Recovery, Role::Primary, None),
        (
            Proposer::Recovery,
            Role::Recovery,
            Some(is_auth_unauthorized_error),
        ),
        (Proposer::Recovery, Role::Confirmation, None),
    ];

    for (proposer, role, error_assertion_function) in test_vectors {
        let mut test_runner = AccessControllerTestRunner::new(Some(10));
        test_runner
            .initiate_recovery(
                proposer.into(),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                Some(10),
            )
            .expect_commit_success();

        // Act
        let receipt = test_runner.quick_confirm_recovery(
            role,
            proposer.into(),
            rule!(require(RADIX_TOKEN)),
            rule!(require(RADIX_TOKEN)),
            rule!(require(RADIX_TOKEN)),
            Some(10),
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

//=============
// State Tests
//=============

mod normal_operations_with_primary_unlocked {
    use super::*;

    const TIMED_RECOVERY_DELAY_IN_MINUTES: Option<u32> = Some(10);

    fn setup_environment() -> AccessControllerTestRunner {
        AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_MINUTES)
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors = [
            (Role::Primary, None),
            (Role::Recovery, Some(is_auth_unauthorized_error)),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] =
            [(Role::Primary, None), (Role::Recovery, None)];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Recovery, // As role
                Role::Primary,  // Proposer
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_recovery_exists_for_proposer_error),
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
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, Some(is_no_timed_recoveries_found_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (
                Role::Primary,
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Recovery,
                Some(is_no_recovery_exists_for_proposer_error),
            ),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(role);

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
    pub fn stop_timed_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, Some(is_no_timed_recoveries_found_error)),
            (Role::Recovery, Some(is_no_timed_recoveries_found_error)),
            (Role::Confirmation, Some(is_no_timed_recoveries_found_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.stop_timed_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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

    const TIMED_RECOVERY_DELAY_IN_MINUTES: Option<u32> = Some(10);

    fn setup_environment() -> AccessControllerTestRunner {
        let mut test_runner = AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_MINUTES);
        test_runner
            .lock_primary_role(Role::Recovery)
            .expect_commit_success();
        test_runner
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_operation_requires_unlocked_primary_role_error),
            ),
            (Role::Recovery, Some(is_auth_unauthorized_error)),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] =
            [(Role::Primary, None), (Role::Recovery, None)];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Recovery, // As role
                Role::Primary,  // Proposer
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_recovery_exists_for_proposer_error),
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
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, Some(is_no_timed_recoveries_found_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (
                Role::Primary,
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Recovery,
                Some(is_no_recovery_exists_for_proposer_error),
            ),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(role);

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
    pub fn stop_timed_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, Some(is_no_timed_recoveries_found_error)),
            (Role::Recovery, Some(is_no_timed_recoveries_found_error)),
            (Role::Confirmation, Some(is_no_timed_recoveries_found_error)),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.stop_timed_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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

    const TIMED_RECOVERY_DELAY_IN_MINUTES: Option<u32> = Some(10);

    fn setup_environment() -> AccessControllerTestRunner {
        let mut test_runner = AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_MINUTES);
        test_runner
            .initiate_recovery(
                Role::Recovery,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
            )
            .expect_commit_success();
        test_runner
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, None),
            (Role::Recovery, Some(is_auth_unauthorized_error)),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (Role::Primary, None),
            (
                Role::Recovery,
                Some(is_recovery_already_exists_for_proposer_error),
            ),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_recovery_exists_for_proposer_error),
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
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (
                Role::Recovery,
                Some(is_timed_recovery_delay_has_not_elapsed_error),
            ),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (
                Role::Primary,
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (Role::Recovery, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(role);

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
    pub fn stop_timed_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, None),
            (Role::Recovery, None),
            (Role::Confirmation, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.stop_timed_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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

    const TIMED_RECOVERY_DELAY_IN_MINUTES: Option<u32> = Some(10);

    fn setup_environment() -> AccessControllerTestRunner {
        let mut test_runner = AccessControllerTestRunner::new(TIMED_RECOVERY_DELAY_IN_MINUTES);
        test_runner
            .lock_primary_role(Role::Recovery)
            .expect_commit_success();
        test_runner
            .initiate_recovery(
                Role::Recovery,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
            )
            .expect_commit_success();
        test_runner
    }

    #[test]
    pub fn create_proof() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (
                Role::Primary,
                Some(is_operation_requires_unlocked_primary_role_error),
            ),
            (Role::Recovery, Some(is_auth_unauthorized_error)),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
    pub fn initiate_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (Role::Primary, None),
            (
                Role::Recovery,
                Some(is_recovery_already_exists_for_proposer_error),
            ),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.initiate_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (Role::Recovery, None),
            (Role::Confirmation, Some(is_auth_unauthorized_error)),
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
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (
                Role::Confirmation, // As role
                Role::Primary,      // Proposer
                Some(is_no_recovery_exists_for_proposer_error),
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
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (Role::Primary, Some(is_auth_unauthorized_error)),
            (
                Role::Recovery,
                Some(is_timed_recovery_delay_has_not_elapsed_error),
            ),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.timed_confirm_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 2] = [
            (
                Role::Primary,
                Some(is_no_recovery_exists_for_proposer_error),
            ),
            (Role::Recovery, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.cancel_recovery_attempt(role);

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
    pub fn stop_timed_recovery() {
        // Arrange
        let test_vectors: [(Role, Option<ErrorCheckFunction>); 3] = [
            (Role::Primary, None),
            (Role::Recovery, None),
            (Role::Confirmation, None),
        ];

        for (role, error_assertion_function) in test_vectors {
            let mut test_runner = setup_environment();

            // Act
            let receipt = test_runner.stop_timed_recovery(
                role,
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                rule!(require(RADIX_TOKEN)),
                TIMED_RECOVERY_DELAY_IN_MINUTES,
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

fn is_auth_unauthorized_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
    )
}

fn is_operation_requires_unlocked_primary_role_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::OperationRequiresUnlockedPrimaryRole
        ))
    )
}

fn is_recovery_already_exists_for_proposer_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::RecoveryAlreadyExistsForProposer { .. }
        ))
    )
}

fn is_no_recovery_exists_for_proposer_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::NoRecoveryExistsForProposer { .. }
        ))
    )
}

fn is_no_timed_recoveries_found_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::NoTimedRecoveriesFound
        ))
    )
}

fn is_timed_recovery_delay_has_not_elapsed_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::TimedRecoveryDelayHasNotElapsed
        ))
    )
}

fn is_recovery_proposal_mismatch_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
            AccessControllerError::RecoveryProposalMismatch { .. }
        ))
    )
}

#[allow(dead_code)]
struct AccessControllerTestRunner {
    pub test_runner: TestRunner,

    pub account: (ComponentAddress, PublicKey),

    pub access_controller_address: ComponentAddress,
    pub primary_role_badge: ResourceAddress,
    pub recovery_role_badge: ResourceAddress,
    pub confirmation_role_badge: ResourceAddress,

    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[allow(dead_code)]
impl AccessControllerTestRunner {
    pub fn new(timed_recovery_delay_in_minutes: Option<u32>) -> Self {
        let mut test_runner = TestRunner::builder().build();

        // Creating a new account - this is where the badges will be held
        let (public_key, _, account) = test_runner.new_account(false);

        // Creating the resource to be protected
        let controlled_asset = test_runner.create_fungible_resource(1.into(), 0, account);

        // Creating three badges for the three roles.
        let primary_role_badge = test_runner.create_fungible_resource(1.into(), 0, account);
        let recovery_role_badge = test_runner.create_fungible_resource(1.into(), 0, account);
        let confirmation_role_badge = test_runner.create_fungible_resource(1.into(), 0, account);

        // Creating the access controller component
        let manifest = ManifestBuilder::new()
            .lock_fee(account, 10.into())
            .withdraw_from_account(account, controlled_asset, 1.into())
            .take_from_worktop(controlled_asset, |builder, bucket| {
                builder.create_access_controller(
                    bucket,
                    rule!(require(primary_role_badge)),
                    rule!(require(recovery_role_badge)),
                    rule!(require(confirmation_role_badge)),
                    timed_recovery_delay_in_minutes,
                )
            })
            .build();
        let receipt = test_runner.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        let access_controller_address = receipt.expect_commit(true).new_component_addresses()[0];

        Self {
            test_runner,
            account: (account, public_key.into()),

            access_controller_address,
            primary_role_badge,
            recovery_role_badge,
            confirmation_role_badge,

            timed_recovery_delay_in_minutes,
        }
    }

    pub fn create_proof(&mut self, as_role: Role) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                "create_proof",
                to_manifest_value(&AccessControllerCreateProofInput {}),
            )
            .pop_from_auth_zone(|builder, _| builder)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn initiate_recovery(
        &mut self,
        as_role: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> TransactionReceipt {
        let method_name = match as_role {
            Role::Primary => ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
            Role::Recovery => ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT,
            Role::Confirmation => panic!("Confirmation Role can't initiate recovery!"),
        };

        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                method_name,
                to_manifest_value(&AccessControllerInitiateRecoveryAsPrimaryInput {
                    rule_set: RuleSet {
                        primary_role: proposed_primary_role,
                        recovery_role: proposed_recovery_role,
                        confirmation_role: proposed_confirmation_role,
                    },
                    timed_recovery_delay_in_minutes,
                }),
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
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> TransactionReceipt {
        let proposer = match proposer {
            Role::Primary => Proposer::Primary,
            Role::Recovery => Proposer::Recovery,
            Role::Confirmation => panic!("Confirmation is not a valid proposer"),
        };

        let method_name = match proposer {
            Proposer::Primary => {
                ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT
            }
            Proposer::Recovery => {
                ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
            }
        };

        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                method_name,
                to_manifest_value(
                    &AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
                        rule_set: RuleSet {
                            primary_role: proposed_primary_role,
                            recovery_role: proposed_recovery_role,
                            confirmation_role: proposed_confirmation_role,
                        },
                        timed_recovery_delay_in_minutes,
                    },
                ),
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
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT,
                to_manifest_value(&AccessControllerTimedConfirmRecoveryInput {
                    rule_set: RuleSet {
                        primary_role: proposed_primary_role,
                        recovery_role: proposed_recovery_role,
                        confirmation_role: proposed_confirmation_role,
                    },
                    timed_recovery_delay_in_minutes,
                }),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn cancel_recovery_attempt(&mut self, as_role: Role) -> TransactionReceipt {
        let method_name = match as_role {
            Role::Primary => ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
            Role::Recovery => ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
            Role::Confirmation => panic!("No method for the given role"),
        };

        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                method_name,
                to_manifest_value(&AccessControllerCancelPrimaryRoleRecoveryProposalInput),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_primary_role(&mut self, as_role: Role) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                "lock_primary_role",
                to_manifest_value(&AccessControllerLockPrimaryRoleInput {}),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn unlock_primary_role(&mut self, as_role: Role) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                "unlock_primary_role",
                to_manifest_value(&AccessControllerUnlockPrimaryRoleInput {}),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn stop_timed_recovery(
        &mut self,
        as_role: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_role)
            .call_method(
                self.access_controller_address,
                "stop_timed_recovery",
                to_manifest_value(&AccessControllerStopTimedRecoveryInput {
                    rule_set: RuleSet {
                        primary_role: proposed_primary_role,
                        recovery_role: proposed_recovery_role,
                        confirmation_role: proposed_confirmation_role,
                    },
                    timed_recovery_delay_in_minutes,
                }),
            )
            .build();
        self.execute_manifest(manifest)
    }

    fn execute_manifest(&mut self, manifest: TransactionManifest) -> TransactionReceipt {
        self.test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&self.account.1)],
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

    fn push_time_forward(&mut self, minutes: i64) {
        let current_time = self.test_runner.get_current_time(TimePrecision::Minute);
        let new_time = current_time.add_minutes(minutes).unwrap();
        self.test_runner
            .set_current_time(new_time.seconds_since_unix_epoch * 1000);
    }
}
