use radix_engine::errors::{ModuleError, RuntimeError, SystemError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::{
    AuthAddresses, ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
    ACCESS_RULES_SET_GROUP_MUTABILITY_IDENT, ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
    ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::TransactionManifest;

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_can_not_be_mutated_when_locked() {
    // Arrange
    let access_rules = AccessRulesConfig::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(deny_all),
        )
        .default(rule!(allow_all), rule!(deny_all));
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.set_method_auth(1, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_cant_be_mutated_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = AccessRulesConfig::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all));
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.set_method_auth(1, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_cant_be_locked_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = AccessRulesConfig::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all));
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.lock_method_auth(1, "deposit_funds");

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_can_be_mutated_when_required_proofs_are_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = AccessRulesConfig::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all));
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id);

    // Act
    let receipt = test_runner.set_method_auth(1, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_commit_success();
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_can_be_locked_when_required_proofs_are_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = AccessRulesConfig::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all));
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id);

    // Act
    let receipt = test_runner.lock_method_auth(1, "deposit_funds");

    // Assert
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.set_method_auth(1, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn method_that_falls_within_default_cant_have_its_auth_mutated() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = AccessRulesConfig::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all));
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id.clone());

    test_runner.lock_default_auth(1);

    // Act
    let receipt = test_runner.set_method_auth(1, "borrow_funds", rule!(deny_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

fn component_access_rules_can_be_mutated_through_manifest(to_rule: AccessRule) {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = AccessRulesConfig::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .method(
            "borrow_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all));
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id.clone());

    // Act
    let receipt = test_runner.execute_manifest(
        MutableAccessRulesTestRunner::manifest_builder()
            .set_method_access_rule(
                test_runner.component_address.into(),
                MethodKey::new(TypedModuleId::ObjectState, "borrow_funds"),
                to_rule,
            )
            .build(),
    );

    // Assert
    receipt.expect_commit_success();
    let receipt = test_runner.borrow_funds();
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
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
fn user_can_not_mutate_auth_on_methods_that_control_auth() {
    // Arrange
    for access_rule_key in [
        MethodKey::new(
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
        ),
        MethodKey::new(
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_MUTABILITY_IDENT,
        ),
        MethodKey::new(
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
        ),
        MethodKey::new(
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
        ),
    ] {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
        let public_key = private_key.public_key();
        let virtual_badge_non_fungible_global_id =
            NonFungibleGlobalId::from_public_key(&public_key);

        let access_rules: AccessRulesConfig = manifest_args!(
            HashMap::<MethodKey, AccessRuleEntry>::new(),
            HashMap::<MethodKey, AccessRuleEntry>::new(),
            HashMap::<String, AccessRule>::new(),
            AccessRuleEntry::AccessRule(AccessRule::AllowAll),
            HashMap::<MethodKey, AccessRule>::new(),
            HashMap::<String, AccessRule>::new(),
            AccessRuleEntry::AccessRule(AccessRule::AllowAll)
        );

        let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
        test_runner.add_initial_proof(virtual_badge_non_fungible_global_id.clone());

        // Act
        let receipt = test_runner.execute_manifest(
            MutableAccessRulesTestRunner::manifest_builder()
                .set_method_access_rule(
                    test_runner.component_address.into(),
                    access_rule_key,
                    rule!(deny_all),
                )
                .build(),
        );

        // Assert
        receipt.expect_specific_failure(|e| {
            matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
        });
    }
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

struct MutableAccessRulesTestRunner {
    test_runner: TestRunner,
    component_address: ComponentAddress,
    initial_proofs: BTreeSet<NonFungibleGlobalId>,
}

impl MutableAccessRulesTestRunner {
    const BLUEPRINT_NAME: &'static str = "MutableAccessRulesComponent";

    pub fn new(access_rules: AccessRulesConfig) -> Self {
        let mut test_runner = TestRunner::builder().build();
        let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                Self::BLUEPRINT_NAME,
                "new",
                manifest_args!(access_rules),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
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

    pub fn set_method_auth(
        &mut self,
        index: usize,
        method_name: &str,
        access_rule: AccessRule,
    ) -> TransactionReceipt {
        let args = manifest_args!(index, method_name.to_string(), access_rule);
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "set_method_auth", args)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn _set_default_auth(
        &mut self,
        index: usize,
        access_rule: AccessRule,
    ) -> TransactionReceipt {
        let args = manifest_args!(index, access_rule);
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "set_default", args)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_method_auth(&mut self, index: usize, method_name: &str) -> TransactionReceipt {
        let args = manifest_args!(index, method_name.to_string());
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "lock_method_auth", args)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_default_auth(&mut self, index: usize) -> TransactionReceipt {
        let args = manifest_args!(index);
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "lock_default_auth", args)
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

    pub fn execute_manifest(&mut self, manifest: TransactionManifest) -> TransactionReceipt {
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, self.initial_proofs.clone())
    }
}
