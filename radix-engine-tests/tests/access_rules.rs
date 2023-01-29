use radix_engine::engine::{ApplicationError, AuthError, ModuleError, RuntimeError};
use radix_engine::model::{AccessRulesChainError, AuthZoneError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::rule;
use scrypto::component::ComponentAccessRules;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;
use transaction::signing::EcdsaSecp256k1PrivateKey;

#[test]
fn scrypto_methods_and_functions_should_be_able_to_return_access_rules_pointers() {
    // Arrange
    let access_rules = vec![
        AccessRules::new()
            .method(
                "deposit_funds",
                rule!(require(RADIX_TOKEN)),
                rule!(deny_all),
            )
            .default(rule!(allow_all), rule!(deny_all)),
        AccessRules::new()
            .method(
                "deposit_funds",
                rule!(require(ECDSA_SECP256K1_TOKEN)),
                rule!(deny_all),
            )
            .default(rule!(allow_all), rule!(deny_all)),
    ];
    for call in [/*Call::Method, */ Call::Function] {
        let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

        // Act
        let read_access_rules = test_runner.access_rules_chain(call);

        // Assert

        assert_eq!(
            access_rules.len() + 1, // TODO: The system adds a layer of access rules, is this the right abstraction?
            read_access_rules.len(),
        )
    }
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn component_access_rules_may_be_changed_within_a_scrypto_method() {
    // Arrange
    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(allow_all),
        )
        .default(rule!(allow_all), rule!(deny_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.deposit_funds();

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });

    // Act
    let receipt = test_runner.set_method_auth(1, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.deposit_funds();

    // Assert
    receipt.expect_commit_success();
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_can_not_be_mutated_when_locked() {
    // Arrange
    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(deny_all),
        )
        .default(rule!(allow_all), rule!(deny_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.set_method_auth(1, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                AccessRulesChainError::Unauthorized(..)
            ))
        )
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_cant_be_mutated_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.set_method_auth(1, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                AccessRulesChainError::Unauthorized(..)
            ))
        )
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_cant_be_locked_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.lock_method_auth(1, "deposit_funds");

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                AccessRulesChainError::Unauthorized(..)
            ))
        )
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn access_rules_method_auth_can_be_mutated_when_required_proofs_are_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all))];
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

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all))];
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
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                AccessRulesChainError::Unauthorized(..)
            ))
        )
    });
}

#[test]
#[ignore] // Unignore once self auth supported in scrypto layer
fn method_that_falls_within_default_cant_have_its_auth_mutated() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            rule!(require(virtual_badge_non_fungible_global_id.clone())),
        )
        .default(rule!(allow_all), rule!(deny_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id.clone());

    test_runner.lock_default_auth(1);

    // Act
    let receipt = test_runner.set_method_auth(1, "borrow_funds", rule!(deny_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                AccessRulesChainError::Unauthorized(..)
            ))
        )
    });
}

#[test]
fn component_access_rules_can_be_mutated_through_manifest_native_call() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
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
        .default(rule!(allow_all), rule!(deny_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_global_id.clone());

    // Act
    let receipt = test_runner.execute_manifest(
        MutableAccessRulesTestRunner::manifest_builder()
            .set_method_access_rule(
                GlobalAddress::Component(test_runner.component_address),
                1,
                AccessRuleKey::ScryptoMethod("borrow_funds".to_string()),
                rule!(deny_all),
            )
            .build(),
    );

    // Assert
    receipt.expect_commit_success();
    let receipt = test_runner.borrow_funds();
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
        )
    });
}

#[test]
fn user_can_not_mutate_auth_on_methods_that_control_auth() {
    // Arrange
    for method in [
        AccessRulesChainFn::GetLength,
        AccessRulesChainFn::SetGroupAccessRule,
        AccessRulesChainFn::SetGroupMutability,
        AccessRulesChainFn::SetMethodAccessRule,
        AccessRulesChainFn::SetMethodMutability,
    ] {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
        let public_key = private_key.public_key();
        let virtual_badge_non_fungible_global_id =
            NonFungibleGlobalId::from_public_key(&public_key);

        let access_rules = vec![scrypto_decode::<AccessRules>(&args!(
            HashMap::<AccessRuleKey, AccessRuleEntry>::new(),
            HashMap::<String, AccessRule>::new(),
            AccessRule::AllowAll,
            HashMap::<AccessRuleKey, AccessRule>::new(),
            HashMap::<String, AccessRule>::new(),
            AccessRule::AllowAll
        ))
        .unwrap()];

        let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
        test_runner.add_initial_proof(virtual_badge_non_fungible_global_id.clone());

        // Act
        let receipt = test_runner.execute_manifest(
            MutableAccessRulesTestRunner::manifest_builder()
                .set_method_access_rule(
                    GlobalAddress::Component(test_runner.component_address),
                    1,
                    AccessRuleKey::Native(NativeFn::AccessRulesChain(method)),
                    rule!(deny_all),
                )
                .build(),
        );

        // Assert
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(..))
            )
        });
    }
}

#[test]
fn assert_access_rule_through_manifest_when_not_fulfilled_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (public_key, _, _account_component) = test_runner.new_account(false);

    let manifest = ManifestBuilder::new()
        .assert_access_rule(rule!(require(RADIX_TOKEN)))
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        [NonFungibleGlobalId::from_public_key(&public_key)].into(),
    );

    // Assert
    receipt.expect_specific_failure(|error: &RuntimeError| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::AuthZoneError(
                AuthZoneError::AssertAccessRuleError(..)
            ))
        )
    })
}

#[test]
fn assert_access_rule_through_manifest_when_fulfilled_succeeds() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (public_key, _, account_component) = test_runner.new_account(false);

    let manifest = ManifestBuilder::new()
        .create_proof_from_account(account_component, RADIX_TOKEN)
        .assert_access_rule(rule!(require(RADIX_TOKEN)))
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        [NonFungibleGlobalId::from_public_key(&public_key)].into(),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn assert_access_rule_through_component_when_not_fulfilled_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (public_key, _, account_component) = test_runner.new_account(false);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "AssertAccessRule".into(), "new", args!())
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)].into(),
        );
        receipt.expect_commit_success();

        receipt.new_component_addresses()[0]
    };

    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account_component, RADIX_TOKEN)
        .call_method(
            component_address,
            "assert_access_rule",
            args!(rule!(require(RADIX_TOKEN)), Vec::<ManifestBucket>::new()),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        [NonFungibleGlobalId::from_public_key(&public_key)].into(),
    );

    // Assert
    receipt.expect_specific_failure(|error: &RuntimeError| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::AuthZoneError(
                AuthZoneError::AssertAccessRuleError(..)
            ))
        )
    })
}

#[test]
fn assert_access_rule_through_component_when_fulfilled_succeeds() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (public_key, _, account_component) = test_runner.new_account(false);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "AssertAccessRule".into(), "new", args!())
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)].into(),
        );
        receipt.expect_commit_success();

        receipt.new_component_addresses()[0]
    };

    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account_component, RADIX_TOKEN)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket| {
            builder.call_method(
                component_address,
                "assert_access_rule",
                args!(rule!(require(RADIX_TOKEN)), vec![bucket]),
            )
        })
        .call_method(
            account_component,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        [NonFungibleGlobalId::from_public_key(&public_key)].into(),
    );

    // Assert
    receipt.expect_commit_success();
}

struct MutableAccessRulesTestRunner {
    test_runner: TestRunner,
    package_address: PackageAddress,
    component_address: ComponentAddress,
    initial_proofs: Vec<NonFungibleGlobalId>,
}

impl MutableAccessRulesTestRunner {
    const BLUEPRINT_NAME: &'static str = "MutableAccessRulesComponent";

    pub fn new(access_rules: Vec<AccessRules>) -> Self {
        let mut test_runner = TestRunner::builder().build();
        let package_address = test_runner.compile_and_publish("./tests/blueprints/access_rules");

        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                Self::BLUEPRINT_NAME,
                "new",
                args!(access_rules),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
        let component_address = receipt.new_component_addresses()[0];

        Self {
            test_runner,
            package_address,
            component_address,
            initial_proofs: Vec::new(),
        }
    }

    pub fn add_initial_proof(&mut self, initial_proof: NonFungibleGlobalId) {
        self.initial_proofs.push(initial_proof);
    }

    pub fn access_rules_chain(&mut self, call: Call) -> Vec<ComponentAccessRules> {
        let manifest = match call {
            /*
            Call::Method => Self::manifest_builder()
                .call_method(self.component_address, "access_rules_method", args!())
                .build(),
             */
            Call::Function => Self::manifest_builder()
                .call_function(
                    self.package_address,
                    Self::BLUEPRINT_NAME,
                    "access_rules_function",
                    args!(self.component_address),
                )
                .build(),
        };

        self.execute_manifest(manifest).output(1)
    }

    pub fn set_method_auth(
        &mut self,
        index: usize,
        method_name: &str,
        access_rule: AccessRule,
    ) -> TransactionReceipt {
        let args = args!(index, method_name.to_string(), access_rule);
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
        let args = args!(index, access_rule);
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "set_default", args)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_method_auth(&mut self, index: usize, method_name: &str) -> TransactionReceipt {
        let args = args!(index, method_name.to_string());
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "lock_method_auth", args)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn lock_default_auth(&mut self, index: usize) -> TransactionReceipt {
        let args = args!(index);
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "lock_default_auth", args)
            .build();
        self.execute_manifest(manifest)
    }

    pub fn deposit_funds(&mut self) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "deposit_funds", args!())
            .build();
        self.execute_manifest(manifest)
    }

    pub fn borrow_funds(&mut self) -> TransactionReceipt {
        let manifest = Self::manifest_builder()
            .call_method(self.component_address, "borrow_funds", args!())
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

enum Call {
    //Method,
    Function,
}
