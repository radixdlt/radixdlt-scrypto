use radix_engine::errors::{RuntimeError, SystemError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::rule;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::*;
use transaction::signing::secp256k1::Secp256k1PrivateKey;

#[test]
fn can_call_public_function() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

    // Act
    let receipt = test_runner.call_function(
        package_address,
        "FunctionAccessRules",
        "public_function",
        manifest_args!(),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_call_protected_function_without_auth() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

    // Act
    let receipt = test_runner.call_function(
        package_address,
        "FunctionAccessRules",
        "protected_function",
        manifest_args!(),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn can_call_protected_function_with_auth() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");
    let (key, _priv, account) = test_runner.new_account(true);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(account, RADIX_TOKEN)
        .call_function(
            package_address,
            "FunctionAccessRules",
            "protected_function",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, [NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn access_rules_method_auth_cannot_be_mutated_when_locked() {
    // Arrange
    let mut roles = RolesInit::new();
    roles.define_immutable_role("deposit_funds_auth_update", rule!(allow_all));
    roles.define_mutable_role("borrow_funds_auth", rule!(allow_all));
    roles.define_immutable_role("deposit_funds_auth", rule!(require(RADIX_TOKEN)));
    let mut test_runner = MutableAccessRulesTestRunner::new(roles);

    // Act
    let receipt = test_runner.set_role_rule(RoleKey::new("deposit_funds_auth"), rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
        )
    });
}

#[test]
fn access_rules_method_auth_cant_be_mutated_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = Secp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);
    let mut test_runner = MutableAccessRulesTestRunner::new_with_owner(rule!(require(
        virtual_badge_non_fungible_global_id.clone()
    )));

    // Act
    let receipt = test_runner.set_role_rule(RoleKey::new("borrow_funds_auth"), rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
        )
    });
}

#[test]
fn access_rules_method_auth_cant_be_locked_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = Secp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);
    let mut test_runner = MutableAccessRulesTestRunner::new_with_owner(rule!(require(
        virtual_badge_non_fungible_global_id.clone()
    )));

    // Act
    let receipt = test_runner.lock_role(RoleKey::new("borrow_funds_auth"));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
        )
    });
}

#[test]
fn access_rules_method_auth_can_be_mutated_when_required_proofs_are_present() {
    // Arrange
    let private_key = Secp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);
    let mut test_runner = MutableAccessRulesTestRunner::new_with_owner(rule!(require(
        virtual_badge_non_fungible_global_id.clone()
    )));

    // Act
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id);
    let receipt = test_runner.set_role_rule(RoleKey::new("borrow_funds_auth"), rule!(allow_all));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn access_rules_method_auth_can_be_locked_when_required_proofs_are_present() {
    // Arrange
    let private_key = Secp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);
    let mut test_runner = MutableAccessRulesTestRunner::new_with_owner(rule!(require(
        virtual_badge_non_fungible_global_id.clone()
    )));
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id);

    // Act
    let receipt = test_runner.lock_role(RoleKey::new("borrow_funds_auth"));

    // Assert
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.set_role_rule(RoleKey::new("borrow_funds_auth"), rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
        )
    });
}

fn component_access_rules_can_be_mutated_through_manifest(to_rule: AccessRule) {
    // Arrange
    let private_key = Secp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);
    let mut test_runner = MutableAccessRulesTestRunner::new_with_owner(rule!(require(
        virtual_badge_non_fungible_global_id.clone()
    )));
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id.clone());

    // Act
    let receipt = test_runner.execute_manifest(
        MutableAccessRulesTestRunner::manifest_builder()
            .update_role(
                test_runner.component_address.into(),
                ObjectModuleId::Main,
                RoleKey::new("borrow_funds_auth"),
                to_rule,
            )
            .build(),
    );

    // Assert
    receipt.expect_commit_success();
    let receipt = test_runner.borrow_funds();
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
        )
    });
}

#[test]
fn component_access_rules_can_be_mutated_to_deny_all_through_manifest() {
    component_access_rules_can_be_mutated_through_manifest(rule!(deny_all));
}

#[test]
fn component_access_rules_can_be_mutated_to_fungible_resource_through_manifest() {
    component_access_rules_can_be_mutated_through_manifest(rule!(require(RADIX_TOKEN)));
}

#[test]
fn component_access_rules_can_be_mutated_to_non_fungible_resource_through_manifest() {
    let non_fungible_global_id = AuthAddresses::system_role();
    component_access_rules_can_be_mutated_through_manifest(rule!(require(non_fungible_global_id)));
}

#[test]
fn assert_access_rule_through_component_when_not_fulfilled_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");
    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "AssertAccessRule".into(),
                "new",
                manifest_args!(),
            )
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(manifest, []);
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "assert_access_rule",
            manifest_args!(rule!(require(RADIX_TOKEN))),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, []);

    // Assert
    receipt.expect_specific_failure(|error: &RuntimeError| {
        matches!(
            error,
            RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
        )
    })
}

#[test]
fn assert_access_rule_through_component_when_fulfilled_succeeds() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "AssertAccessRule".into(),
                "new",
                manifest_args!(),
            )
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let manifest = ManifestBuilder::new()
        .create_proof_from_account(account, RADIX_TOKEN)
        .call_method(
            component_address,
            "assert_access_rule",
            manifest_args!(rule!(require(RADIX_TOKEN))),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        [NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn update_rule() {
    // Arrange
    let private_key = Secp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);
    let mut test_runner = MutableAccessRulesTestRunner::new_with_owner(rule!(require(
        virtual_badge_non_fungible_global_id.clone()
    )));

    let receipt = test_runner.get_role(RoleKey::new("borrow_funds_auth"));
    let ret = receipt.expect_commit(true).outcome.expect_success();
    assert_eq!(
        ret[1],
        InstructionOutput::CallReturn(
            scrypto_encode(&Some(AccessRule::Protected(AccessRuleNode::ProofRule(
                ProofRule::Require(ResourceOrNonFungible::Resource(RADIX_TOKEN))
            ))))
            .unwrap()
        )
    );

    // Act, update rule
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id);
    let receipt = test_runner.set_role_rule(RoleKey::new("borrow_funds_auth"), rule!(allow_all));
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.get_role(RoleKey::new("borrow_funds_auth"));

    // Assert
    let ret = receipt.expect_commit(true).outcome.expect_success();
    assert_eq!(
        ret[1],
        InstructionOutput::CallReturn(scrypto_encode(&Some(AccessRule::AllowAll)).unwrap())
    );
}

#[test]
fn change_lock_owner_role_rules() {
    // Arrange
    let mut test_runner =
        MutableAccessRulesTestRunner::new_with_owner_role(OwnerRole::Updatable(rule!(allow_all)));

    // Act: verify if lock owner role is possible
    let receipt = test_runner.lock_owner_role();
    receipt.expect_commit(true).outcome.expect_success();

    // Act: change lock owner role rule to deny all
    let receipt = test_runner.set_owner_role(rule!(deny_all));
    receipt.expect_commit_success();

    // Act: verify if lock owner role is not possible  now
    let receipt = test_runner.lock_owner_role();
    receipt.expect_specific_failure(|error: &RuntimeError| {
        matches!(
            error,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                _
            )))
        )
    })
}

struct MutableAccessRulesTestRunner {
    test_runner: TestRunner,
    component_address: ComponentAddress,
    initial_proofs: BTreeSet<NonFungibleGlobalId>,
}

impl MutableAccessRulesTestRunner {
    const BLUEPRINT_NAME: &'static str = "MutableAccessRulesComponent";

    pub fn create_component(roles: RolesInit, test_runner: &mut TestRunner) -> TransactionReceipt {
        let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                Self::BLUEPRINT_NAME,
                "new",
                manifest_args!(roles),
            )
            .build();
        test_runner.execute_manifest_ignoring_fee(manifest, vec![])
    }

    pub fn create_component_with_owner(
        owner_role: OwnerRole,
        test_runner: &mut TestRunner,
    ) -> TransactionReceipt {
        let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                Self::BLUEPRINT_NAME,
                "new_with_owner",
                manifest_args!(owner_role),
            )
            .build();
        test_runner.execute_manifest_ignoring_fee(manifest, vec![])
    }

    pub fn new_with_owner(update_access_rule: AccessRule) -> Self {
        let mut test_runner = TestRunner::builder().build();
        let receipt = Self::create_component_with_owner(
            OwnerRole::Fixed(update_access_rule),
            &mut test_runner,
        );
        let component_address = receipt.expect_commit(true).new_component_addresses()[0];

        Self {
            test_runner,
            component_address,
            initial_proofs: BTreeSet::new(),
        }
    }

    pub fn new_with_owner_role(owner_role: OwnerRole) -> Self {
        let mut test_runner = TestRunner::builder().build();
        let receipt = Self::create_component_with_owner(owner_role, &mut test_runner);
        let component_address = receipt.expect_commit(true).new_component_addresses()[0];

        Self {
            test_runner,
            component_address,
            initial_proofs: BTreeSet::new(),
        }
    }

    pub fn new(roles: RolesInit) -> Self {
        let mut test_runner = TestRunner::builder().build();
        let receipt = Self::create_component(roles, &mut test_runner);
        let component_address = receipt.expect_commit(true).new_component_addresses()[0];

        Self {
            test_runner,
            component_address,
            initial_proofs: BTreeSet::new(),
        }
    }

    pub fn add_initial_proof(&mut self, initial_proof: NonFungibleGlobalId) {
        self.initial_proofs.insert(initial_proof);
    }

    pub fn set_role_rule(
        &mut self,
        role_key: RoleKey,
        access_rule: AccessRule,
    ) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .update_role(
                self.component_address.into(),
                ObjectModuleId::Main,
                role_key,
                access_rule,
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn get_role(&mut self, role_key: RoleKey) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .get_role(
                self.component_address.into(),
                ObjectModuleId::Main,
                role_key,
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_role(&mut self, role_key: RoleKey) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .lock_role(
                self.component_address.into(),
                ObjectModuleId::Main,
                role_key,
            )
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_owner_role(&mut self) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .lock_owner_role(self.component_address.into())
            .build();
        self.execute_manifest(manifest)
    }

    pub fn set_owner_role(&mut self, rule: AccessRule) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .set_owner_role(self.component_address.into(), rule)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn borrow_funds(&mut self) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "borrow_funds", manifest_args!())
            .build();
        self.execute_manifest(manifest)
    }

    pub fn manifest_builder() -> ManifestBuilder {
        ManifestBuilder::new()
    }

    pub fn execute_manifest(&mut self, manifest: TransactionManifestV1) -> TransactionReceipt {
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, self.initial_proofs.clone())
    }
}
