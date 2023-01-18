use radix_engine::{
    engine::{ApplicationError, AuthError, ModuleError, RuntimeError},
    model::AccessControllerError,
    transaction::TransactionReceipt,
    types::*,
};
use scrypto_unit::TestRunner;
use transaction::{builder::ManifestBuilder, model::TransactionManifest};

#[test]
pub fn creating_proof_as_primary_during_normal_operations_with_unlocked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.create_proof([Role::Primary].into());

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn creating_proof_as_recovery_during_normal_operations_with_unlocked_primary_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.create_proof([Role::Recovery].into());

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. })),
        )
    });
}

#[test]
pub fn creating_proof_as_confirmation_during_normal_operations_with_unlocked_primary_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.create_proof([Role::Confirmation].into());

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. })),
        )
    });
}

#[test]
pub fn update_timed_recovery_delay_as_primary_during_normal_operations_with_unlocked_primary_succeeds(
) {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.update_timed_recovery_delay([Role::Primary].into(), 5);

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn update_timed_recovery_delay_as_recovery_during_normal_operations_with_unlocked_primary_fails(
) {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.update_timed_recovery_delay([Role::Recovery].into(), 5);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. })),
        )
    });
}

#[test]
pub fn update_timed_recovery_delay_as_confirmation_during_normal_operations_with_unlocked_primary_fails(
) {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.update_timed_recovery_delay([Role::Confirmation].into(), 5);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. })),
        )
    });
}

#[test]
pub fn initiate_recovery_as_primary_during_normal_operations_with_unlocked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    let primary_role = rule!(require(RADIX_TOKEN));
    let recovery_role = rule!(require(RADIX_TOKEN));
    let confirmation_role = rule!(require(RADIX_TOKEN));

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Primary,
        primary_role,
        recovery_role,
        confirmation_role,
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn initiate_recovery_as_recovery_during_normal_operations_with_unlocked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    let primary_role = rule!(require(RADIX_TOKEN));
    let recovery_role = rule!(require(RADIX_TOKEN));
    let confirmation_role = rule!(require(RADIX_TOKEN));

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Recovery,
        primary_role,
        recovery_role,
        confirmation_role,
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn initiate_recovery_as_confirmation_during_normal_operations_with_unlocked_primary_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    let primary_role = rule!(require(RADIX_TOKEN));
    let recovery_role = rule!(require(RADIX_TOKEN));
    let confirmation_role = rule!(require(RADIX_TOKEN));

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Confirmation,
        primary_role,
        recovery_role,
        confirmation_role,
    );

    // Assert
    let asserted_against = AccessRule::Protected(AccessRuleNode::AnyOf(vec![
        AccessRuleNode::ProofRule(ProofRule::Require(
            SoftResourceOrNonFungible::StaticResource(test_runner.primary_role_badge),
        )),
        AccessRuleNode::ProofRule(ProofRule::Require(
            SoftResourceOrNonFungible::StaticResource(test_runner.recovery_role_badge),
        )),
    ]));
    receipt.expect_specific_failure(|error| {
        error.to_owned()
            == RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
                AccessControllerError::RuleAssertionFailed { asserted_against },
            ))
    });
}

#[test]
pub fn lock_primary_role_as_primary_during_normal_operations_with_unlocked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.lock_primary_role(Role::Primary);

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn lock_primary_role_as_recovery_during_normal_operations_with_unlocked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.lock_primary_role(Role::Recovery);

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn lock_primary_role_as_confirmation_during_normal_operations_with_unlocked_primary_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    // Act
    let receipt = test_runner.lock_primary_role(Role::Confirmation);

    // Assert
    let asserted_against = AccessRule::Protected(AccessRuleNode::AnyOf(vec![
        AccessRuleNode::ProofRule(ProofRule::Require(
            SoftResourceOrNonFungible::StaticResource(test_runner.primary_role_badge),
        )),
        AccessRuleNode::ProofRule(ProofRule::Require(
            SoftResourceOrNonFungible::StaticResource(test_runner.recovery_role_badge),
        )),
    ]));
    receipt.expect_specific_failure(|error| {
        error.to_owned()
            == RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
                AccessControllerError::RuleAssertionFailed { asserted_against },
            ))
    });
}

#[test]
pub fn unlock_primary_role_as_primary_during_normal_operations_with_locked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner
        .unlock_primary_role(Role::Primary)
        .expect_commit_success();

    // Act
    let receipt = test_runner.unlock_primary_role(Role::Primary);

    // Assert
    let asserted_against = AccessRule::Protected(AccessRuleNode::AnyOf(vec![
        AccessRuleNode::ProofRule(ProofRule::Require(
            SoftResourceOrNonFungible::StaticResource(test_runner.confirmation_role_badge),
        )),
        AccessRuleNode::ProofRule(ProofRule::Require(
            SoftResourceOrNonFungible::StaticResource(test_runner.recovery_role_badge),
        )),
    ]));
    receipt.expect_specific_failure(|error| {
        error.to_owned()
            == RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
                AccessControllerError::RuleAssertionFailed { asserted_against },
            ))
    });
}

#[test]
pub fn unlock_primary_role_as_recovery_during_normal_operations_with_locked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner
        .unlock_primary_role(Role::Primary)
        .expect_commit_success();

    // Act
    let receipt = test_runner.unlock_primary_role(Role::Recovery);

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn unlock_primary_role_as_confirmation_during_normal_operations_with_locked_primary_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);
    test_runner
        .unlock_primary_role(Role::Primary)
        .expect_commit_success();

    // Act
    let receipt = test_runner.unlock_primary_role(Role::Confirmation);

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn initiate_recovery_as_primary_during_normal_operations_with_locked_primary_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    let primary_role = rule!(require(RADIX_TOKEN));
    let recovery_role = rule!(require(RADIX_TOKEN));
    let confirmation_role = rule!(require(RADIX_TOKEN));

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Primary,
        primary_role,
        recovery_role,
        confirmation_role,
    );

    // Assert
    let asserted_against = AccessRule::Protected(AccessRuleNode::ProofRule(require(
        test_runner.recovery_role_badge,
    )));
    receipt.expect_specific_failure(|error| {
        error.to_owned()
            == RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
                AccessControllerError::RuleAssertionFailed { asserted_against },
            ))
    });
}

#[test]
pub fn initiate_recovery_as_recovery_during_normal_operations_with_locked_primary_succeeds() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    let primary_role = rule!(require(RADIX_TOKEN));
    let recovery_role = rule!(require(RADIX_TOKEN));
    let confirmation_role = rule!(require(RADIX_TOKEN));

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Recovery,
        primary_role,
        recovery_role,
        confirmation_role,
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn initiate_recovery_as_confirmation_during_normal_operations_with_locked_primary_fails() {
    // Arrange
    let mut test_runner = AccessControllerTestRunner::new(10);

    let primary_role = rule!(require(RADIX_TOKEN));
    let recovery_role = rule!(require(RADIX_TOKEN));
    let confirmation_role = rule!(require(RADIX_TOKEN));

    // Act
    let receipt = test_runner.initiate_recovery(
        Role::Confirmation,
        primary_role,
        recovery_role,
        confirmation_role,
    );

    // Assert
    let asserted_against = AccessRule::Protected(AccessRuleNode::ProofRule(require(
        test_runner.recovery_role_badge,
    )));
    receipt.expect_specific_failure(|error| {
        error.to_owned()
            == RuntimeError::ApplicationError(ApplicationError::AccessControllerError(
                AccessControllerError::RuleAssertionFailed { asserted_against },
            ))
    });
}

struct AccessControllerTestRunner {
    pub test_runner: TestRunner,

    pub account: (ComponentAddress, PublicKey),

    pub access_controller_component_address: ComponentAddress,
    pub primary_role_badge: ResourceAddress,
    pub recovery_role_badge: ResourceAddress,
    pub confirmation_role_badge: ResourceAddress,

    pub timed_recovery_delay_in_hours: u16,
}

impl AccessControllerTestRunner {
    pub fn new(timed_recovery_delay_in_hours: u16) -> Self {
        let mut test_runner = TestRunner::new(true);

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
            .withdraw_from_account(account_component, controlled_asset)
            .take_from_worktop(controlled_asset, |builder, bucket| {
                builder.create_access_controller(
                    bucket,
                    rule!(require(primary_role_badge)),
                    rule!(require(confirmation_role_badge)),
                    rule!(require(recovery_role_badge)),
                    timed_recovery_delay_in_hours,
                )
            })
            .build();
        let receipt = test_runner.execute_manifest(
            manifest,
            vec![NonFungibleAddress::from_public_key(&public_key)],
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

    pub fn create_proof(&mut self, as_roles: HashSet<Role>) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_roles)
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
        as_roles: HashSet<Role>,
        timed_recovery_delay_in_hours: u16,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(as_roles)
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
            .manifest_builder(HashSet::from([as_role.clone()]))
            .call_method(
                self.access_controller_component_address,
                "initiate_recovery",
                scrypto_encode(&AccessControllerInitiateRecoveryMethodArgs {
                    proposer: as_role,
                    proposed_primary_role,
                    proposed_recovery_role,
                    proposed_confirmation_role,
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
            .manifest_builder(HashSet::from([as_role.clone()]))
            .call_method(
                self.access_controller_component_address,
                "quick_confirm_recovery",
                scrypto_encode(&AccessControllerQuickConfirmRecoveryMethodArgs {
                    confirmor: as_role,
                    proposer,
                    proposed_primary_role,
                    proposed_recovery_role,
                    proposed_confirmation_role,
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn timed_confirm_recovery(
        &mut self,
        as_role: Role,
        proposer: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(HashSet::from([as_role.clone()]))
            .call_method(
                self.access_controller_component_address,
                "timed_confirm_recovery",
                scrypto_encode(&AccessControllerTimedConfirmRecoveryMethodArgs {
                    confirmor: as_role,
                    proposer,
                    proposed_primary_role,
                    proposed_recovery_role,
                    proposed_confirmation_role,
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn cancel_recovery_attempt(
        &mut self,
        as_role: Role,
        proposer: Role,
        proposed_primary_role: AccessRule,
        proposed_recovery_role: AccessRule,
        proposed_confirmation_role: AccessRule,
    ) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(HashSet::from([as_role.clone()]))
            .call_method(
                self.access_controller_component_address,
                "cancel_recovery_attempt",
                scrypto_encode(&AccessControllerCancelRecoveryAttemptMethodArgs {
                    proposer,
                    proposed_primary_role,
                    proposed_recovery_role,
                    proposed_confirmation_role,
                })
                .unwrap(),
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_primary_role(&mut self, as_role: Role) -> TransactionReceipt {
        let manifest = self
            .manifest_builder(HashSet::from([as_role.clone()]))
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
            .manifest_builder(HashSet::from([as_role.clone()]))
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
            vec![NonFungibleAddress::from_public_key(&self.account.1)],
        )
    }

    fn manifest_builder(&self, roles: HashSet<Role>) -> ManifestBuilder {
        let mut manifest_builder = ManifestBuilder::new();
        for role in roles {
            let resource_address = match role {
                Role::Primary => self.primary_role_badge,
                Role::Recovery => self.recovery_role_badge,
                Role::Confirmation => self.confirmation_role_badge,
            };
            manifest_builder.create_proof_from_account(self.account.0, resource_address);
        }
        manifest_builder
    }
}
