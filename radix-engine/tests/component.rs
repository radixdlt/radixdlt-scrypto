use radix_engine::engine::{
    ApplicationError, AuthError, InterpreterError, KernelError, LockState, ModuleError,
    RuntimeError, ScryptoFnResolvingError, TrackError,
};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::model::AccessRulesError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::api::types::{RENodeId, ScryptoFunctionIdent};
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::{data::*, rule};
use scrypto::component::ComponentAccessRules;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;
use transaction::signing::EcdsaSecp256k1PrivateKey;

#[test]
fn test_component() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/component");

    // Create component
    let manifest1 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "ComponentTest", "create_component", args!())
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();

    // Find the component address from receipt
    let component = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Call functions & methods
    let manifest2 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            args!(component),
        )
        .call_method(component, "get_component_state", args!())
        .call_method(component, "put_component_state", args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt2 = test_runner.execute_manifest(
        manifest2,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt2.expect_commit_success();
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_addr = test_runner.compile_and_publish("./tests/blueprints/component");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_addr,
            "NonExistentBlueprint",
            "create_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::InterpreterError(InterpreterError::InvalidScryptoFunctionInvocation(
            ScryptoFunctionIdent {
                package: ScryptoPackage::Global(package_address),
                blueprint_name,
                ..
            },
            ScryptoFnResolvingError::BlueprintNotFound,
        )) = e
        {
            package_addr.eq(&package_address) && blueprint_name.eq("NonExistentBlueprint")
        } else {
            false
        }
    });
}

#[test]
fn mut_reentrancy_should_not_be_possible() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/component");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_function(package_address, "ReentrantComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_method(component_address, "call_mut_self", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::TrackError(TrackError::SubstateLocked(
                SubstateId(
                    RENodeId::Component(..),
                    SubstateOffset::Component(ComponentOffset::State)
                ),
                LockState::Write
            )))
        )
    });
}

#[test]
fn read_reentrancy_should_be_possible() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/component");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_function(package_address, "ReentrantComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_method(component_address, "call_self", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn read_then_mut_reentrancy_should_not_be_possible() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/component");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_function(package_address, "ReentrantComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_method(component_address, "call_mut_self_2", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::TrackError(TrackError::SubstateLocked(
                SubstateId(
                    RENodeId::Component(..),
                    SubstateOffset::Component(ComponentOffset::State)
                ),
                LockState::Read(1),
            )))
        )
    });
}

#[test]
fn missing_component_address_in_manifest_should_cause_rejection() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let _ = test_runner.compile_and_publish("./tests/blueprints/component");
    let component_address = Bech32Decoder::new(&NetworkDefinition::simulator())
        .validate_and_decode_component_address(
            "component_sim1qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqph4dhmhs42ee03",
        )
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component_address, "get_component_state", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_rejection();
}

#[test]
fn scrypto_methods_and_functions_should_be_able_to_return_access_rules_pointers() {
    // Arrange
    let access_rules = vec![
        AccessRules::new()
            .method("deposit_funds", rule!(require(RADIX_TOKEN)), LOCKED)
            .default(rule!(allow_all)),
        AccessRules::new()
            .method(
                "deposit_funds",
                rule!(require(ECDSA_SECP256K1_TOKEN)),
                LOCKED,
            )
            .default(rule!(allow_all)),
    ];
    for call in [Call::Method, Call::Function] {
        let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

        // Act
        let read_access_rules = test_runner.access_rules(call);

        // Assert
        assert_eq!(access_rules.len(), read_access_rules.len())
    }
}

#[test]
fn component_access_rules_may_be_changed_within_a_scrypto_method() {
    // Arrange
    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(allow_all)),
        )
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.deposit_funds();

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });

    // Act
    let receipt = test_runner.set_method_auth(0, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.deposit_funds();

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn access_rules_method_auth_can_not_be_mutated_when_locked() {
    // Arrange
    let access_rules = vec![AccessRules::new()
        .method("deposit_funds", rule!(require(RADIX_TOKEN)), LOCKED)
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.set_method_auth(0, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesError(
                AccessRulesError::Unauthorized(..)
            ))
        )
    });
}

#[test]
fn access_rules_method_auth_cant_be_mutated_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require(virtual_badge_non_fungible_address.clone()))),
        )
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.set_method_auth(0, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesError(
                AccessRulesError::Unauthorized(..)
            ))
        )
    });
}

#[test]
fn access_rules_method_auth_cant_be_locked_when_required_proofs_are_not_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require(virtual_badge_non_fungible_address.clone()))),
        )
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());

    // Act
    let receipt = test_runner.lock_method_auth(0, "deposit_funds");

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesError(
                AccessRulesError::Unauthorized(..)
            ))
        )
    });
}

#[test]
fn access_rules_method_auth_can_be_mutated_when_required_proofs_are_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require(virtual_badge_non_fungible_address.clone()))),
        )
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_address);

    // Act
    let receipt = test_runner.set_method_auth(0, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn access_rules_method_auth_can_be_locked_when_required_proofs_are_present() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require(virtual_badge_non_fungible_address.clone()))),
        )
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_address);

    // Act
    let receipt = test_runner.lock_method_auth(0, "deposit_funds");

    // Assert
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.set_method_auth(0, "deposit_funds", rule!(allow_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesError(
                AccessRulesError::Unauthorized(..)
            ))
        )
    });
}

#[test]
fn method_that_falls_within_default_cant_have_its_auth_mutated() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require(virtual_badge_non_fungible_address.clone()))),
        )
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_address.clone());

    test_runner.lock_default_auth(0);

    // Act
    let receipt = test_runner.set_method_auth(0, "borrow_funds", rule!(deny_all));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AccessRulesError(
                AccessRulesError::Unauthorized(..)
            ))
        )
    });
}

#[test]
fn component_access_rules_can_be_mutated_through_manifest_native_call() {
    // Arrange
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
    let public_key = private_key.public_key();
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    let access_rules = vec![AccessRules::new()
        .method(
            "deposit_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require(virtual_badge_non_fungible_address.clone()))),
        )
        .method(
            "borrow_funds",
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require(virtual_badge_non_fungible_address.clone()))),
        )
        .default(rule!(allow_all))];
    let mut test_runner = MutableAccessRulesTestRunner::new(access_rules.clone());
    test_runner.add_initial_proof(virtual_badge_non_fungible_address.clone());

    // Act
    let receipt = test_runner.execute_manifest(
        MutableAccessRulesTestRunner::manifest_builder()
            .call_native_method(
                RENodeId::Global(GlobalAddress::Component(test_runner.component_address)),
                &AccessRulesMethod::SetMethodAccessRule.to_string(),
                scrypto_encode(&AccessRulesSetMethodAccessRuleInvocation {
                    receiver: RENodeId::Global(GlobalAddress::Component(
                        test_runner.component_address,
                    )),
                    index: 0,
                    key: AccessRuleKey::ScryptoMethod("borrow_funds".to_string()),
                    rule: rule!(deny_all),
                })
                .unwrap(),
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
fn user_can_not_control_auth_on_methods_that_mutate_auth() {
    // Arrange
    for method in [
        AccessRulesMethod::GetLength,
        AccessRulesMethod::SetGroupAccessRule,
        AccessRulesMethod::SetGroupMutability,
        AccessRulesMethod::SetMethodAccessRule,
        AccessRulesMethod::SetMethodMutability,
    ] {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(709).unwrap();
        let public_key = private_key.public_key();
        let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

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
        test_runner.add_initial_proof(virtual_badge_non_fungible_address.clone());

        // Act
        let receipt = test_runner.execute_manifest(
            MutableAccessRulesTestRunner::manifest_builder()
                .call_native_method(
                    RENodeId::Global(GlobalAddress::Component(test_runner.component_address)),
                    &AccessRulesMethod::SetMethodAccessRule.to_string(),
                    scrypto_encode(&AccessRulesSetMethodAccessRuleInvocation {
                        receiver: RENodeId::Global(GlobalAddress::Component(
                            test_runner.component_address,
                        )),
                        index: 0,
                        key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRules(
                            method,
                        ))),
                        rule: rule!(deny_all),
                    })
                    .unwrap(),
                )
                .build(),
        );

        // Assert
        receipt.expect_commit_success();
        let receipt = test_runner.borrow_funds();
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::AccessRulesError(..))
            )
        });
    }
}

struct MutableAccessRulesTestRunner {
    substate_store: TypedInMemorySubstateStore,
    package_address: PackageAddress,
    component_address: ComponentAddress,
    initial_proofs: Vec<NonFungibleAddress>,
}

impl MutableAccessRulesTestRunner {
    const BLUEPRINT_NAME: &'static str = "MutableAccessRulesComponent";

    pub fn new(access_rules: Vec<AccessRules>) -> Self {
        let mut store = TypedInMemorySubstateStore::with_bootstrap();
        let mut test_runner = TestRunner::new(true, &mut store);
        let package_address = test_runner.compile_and_publish("./tests/blueprints/component");

        let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
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
            substate_store: store,
            package_address,
            component_address,
            initial_proofs: Vec::new(),
        }
    }

    pub fn add_initial_proof(&mut self, initial_proof: NonFungibleAddress) {
        self.initial_proofs.push(initial_proof);
    }

    pub fn access_rules(&mut self, call: Call) -> Vec<ComponentAccessRules> {
        let manifest = match call {
            Call::Method => Self::manifest_builder()
                .call_method(self.component_address, "access_rules_method", args!())
                .build(),
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
        ManifestBuilder::new(&NetworkDefinition::simulator())
    }

    pub fn execute_manifest(&mut self, manifest: TransactionManifest) -> TransactionReceipt {
        TestRunner::new(true, &mut self.substate_store)
            .execute_manifest_ignoring_fee(manifest, self.initial_proofs.clone())
    }
}

enum Call {
    Method,
    Function,
}
