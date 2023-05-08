use arbitrary::{Arbitrary, Unstructured};
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::resource::{FromPublicKey, NonFungibleGlobalId};
#[cfg(feature = "decode_tx_manifest")]
use radix_engine_interface::data::manifest::manifest_decode;
use radix_engine_stores::interface::SubstateDatabase;
use radix_engine_stores::jmt_support::JmtMapper;
use scrypto_unit::{TestRunner, TestRunnerSnapshot};
use strum::EnumCount;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::manifest::ast;
use transaction::model::Instruction;
use transaction::model::TransactionManifest;

const INSTRUCTION_MAX_CNT: u8 = 3;

#[derive(Debug, Clone)]
struct Account {
    public_key: EcdsaSecp256k1PublicKey,
    //_private_key: EcdsaSecp256k1PrivateKey,
    #[allow(unused)]
    address: ComponentAddress,
    #[allow(unused)]
    resources: Vec<ResourceAddress>,
}

pub struct TxFuzzer {
    runner: TestRunner,
    snapshot: TestRunnerSnapshot,
    accounts: Vec<Account>,
    component_addresses: Vec<ComponentAddress>,
    all_resource_addresses: Vec<ResourceAddress>,
    fungible_resource_addresses: Vec<ResourceAddress>,
    non_fungible_resource_addresses: Vec<ResourceAddress>,
    #[allow(unused)]
    package_addresses: Vec<PackageAddress>,
    public_keys: Vec<EcdsaSecp256k1PublicKey>,
}

impl TxFuzzer {
    pub fn new() -> Self {
        let mut runner = TestRunner::builder().without_trace().build();
        let mut component_addresses = vec![runner.faucet_component()];
        let mut all_resource_addresses = vec![
            RADIX_TOKEN,
            ECDSA_SECP256K1_TOKEN,
            EDDSA_ED25519_TOKEN,
            SYSTEM_TOKEN,
            PACKAGE_TOKEN,
            GLOBAL_OBJECT_TOKEN,
            PACKAGE_OWNER_TOKEN,
            VALIDATOR_OWNER_TOKEN,
            IDENTITY_OWNER_TOKEN,
            ACCOUNT_OWNER_TOKEN,
        ];
        let mut non_fungible_resource_addresses = vec![];
        let mut fungible_resource_addresses = vec![];
        let package_addresses = vec![
            PACKAGE_PACKAGE,
            RESOURCE_MANAGER_PACKAGE,
            IDENTITY_PACKAGE,
            EPOCH_MANAGER_PACKAGE,
            CLOCK_PACKAGE,
            ACCOUNT_PACKAGE,
            ACCESS_CONTROLLER_PACKAGE,
            TRANSACTION_PROCESSOR_PACKAGE,
            METADATA_PACKAGE,
            ROYALTY_PACKAGE,
            ACCESS_RULES_PACKAGE,
            GENESIS_HELPER_PACKAGE,
        ];
        let mut public_keys = vec![];
        let accounts: Vec<Account> = (0..2)
            .map(|_| {
                let acc = runner.new_account(false);
                let resources: Vec<ResourceAddress> = vec![
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                    runner.create_non_fungible_resource(acc.2),
                    runner.create_non_fungible_resource(acc.2),
                ];
                all_resource_addresses.append(&mut resources.clone());
                fungible_resource_addresses.append(&mut resources.clone()[0..2].to_vec());
                non_fungible_resource_addresses.append(&mut resources.clone()[2..4].to_vec());
                component_addresses.push(acc.2);
                public_keys.push(acc.0);

                Account {
                    public_key: acc.0,
                    //_private_key: acc.1,
                    address: acc.2,
                    resources,
                }
            })
            .collect();
        let snapshot = runner.create_snapshot();

        Self {
            runner,
            snapshot,
            accounts,
            component_addresses,
            all_resource_addresses,
            fungible_resource_addresses,
            non_fungible_resource_addresses,
            package_addresses,
            public_keys,
        }
    }

    pub fn reset_runner(&mut self) {
        self.runner.restore_snapshot(self.snapshot.clone());
    }

    // pick account from the preallocated pool basing on the input data
    #[cfg(feature = "smart_mutate")]
    fn get_account(&mut self, data: &[u8]) -> Option<ComponentAddress> {
        let len = data.len();
        if len >= 2 && data[len - 2] % 2 == 0 {
            let idx = *data.last().unwrap() as usize % self.accounts.len();
            return Some(self.accounts[idx].address);
        }
        None
    }

    // pick resource from the preallocated pool basing on the input data
    #[cfg(feature = "smart_mutate")]
    fn get_resource(&mut self, data: &[u8]) -> Option<ResourceAddress> {
        let len = data.len();
        if len >= 3 && data[len - 3] % 2 == 0 {
            let account_idx = *data.last().unwrap() as usize % self.accounts.len();
            let resource_idx = data[len - 2] as usize % self.accounts[account_idx].resources.len();
            Some(self.accounts[account_idx].resources[resource_idx])
        } else {
            None
        }
    }

    // Smartly replace some data in the manifest using some preallocated resources.
    // This is to let fuzzing go "deeper" into the manifest instructions and not to reject the
    // transaction on the very early stage
    #[cfg(feature = "smart_mutate")]
    fn smart_mutate_manifest(&mut self, manifest: &mut TransactionManifest) {
        for i in &mut manifest.instructions {
            match i {
                Instruction::CallMethod {
                    component_address, ..
                }
                | Instruction::SetComponentRoyaltyConfig {
                    component_address, ..
                }
                | Instruction::ClaimComponentRoyalty { component_address } => {
                    if let Some(address) = self.get_account(component_address.as_ref()) {
                        *component_address = address;
                    }
                }
                Instruction::TakeFromWorktop { resource_address }
                | Instruction::TakeFromWorktopByAmount {
                    resource_address, ..
                }
                | Instruction::TakeFromWorktopByIds {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContains { resource_address }
                | Instruction::AssertWorktopContainsByAmount {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContainsByIds {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZone { resource_address }
                | Instruction::CreateProofFromAuthZoneByAmount {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZoneByIds {
                    resource_address, ..
                }
                | Instruction::MintFungible {
                    resource_address, ..
                }
                | Instruction::MintNonFungible {
                    resource_address, ..
                }
                | Instruction::MintUuidNonFungible {
                    resource_address, ..
                } => {
                    if let Some(address) = self.get_resource(resource_address.as_ref()) {
                        *resource_address = address;
                    }
                }
                _ => {}
            }
        }
    }

    #[allow(unused)]
    fn get_random_account(
        &mut self,
        unstructured: &mut Unstructured,
    ) -> Result<(Account, ResourceAddress), arbitrary::Error> {
        let account = unstructured.choose(&self.accounts[..])?;
        let resource_address = unstructured.choose(&account.resources[..])?;

        Ok((account.clone(), resource_address.clone()))
    }

    fn get_non_fungible_local_id(
        &mut self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> BTreeSet<NonFungibleLocalId> {
        let vaults = self
            .runner
            .get_component_vaults(component_address, resource_address);
        let mut btree_ids = btreeset![];
        for vault in vaults {
            let vault = self
                .runner
                .substate_db()
                .get_mapped_substate::<JmtMapper, LiquidNonFungibleVault>(
                    &vault,
                    SysModuleId::Object.into(),
                    NonFungibleVaultOffset::LiquidNonFungible.into(),
                )
                .map(|vault| vault.ids);

            vault.map(|ids| {
                let mut substate_iter = self
                    .runner
                    .substate_db()
                    .list_mapped_substates::<JmtMapper>(
                        ids.as_node_id(),
                        SysModuleId::Object.into(),
                    );
                substate_iter.next().map(|(_key, value)| {
                    let id: NonFungibleLocalId = scrypto_decode(value.as_slice()).unwrap();
                    btree_ids.insert(id);
                });
            });
        }
        btree_ids
    }

    fn build_manifest(&mut self, data: &[u8]) -> Result<TransactionManifest, TxStatus> {
        // Arbitrary does not return error if not enough data to construct a full instance of
        // Self. It uses dummy values (zeros) instead.
        // TODO: to consider if this is ok to allow it.
        let mut unstructured = Unstructured::new(&data);

        // Push some random key to the key vector. We will then randomly pick a key from this
        // vector
        self.public_keys
            .push(EcdsaSecp256k1PublicKey::arbitrary(&mut unstructured).unwrap());

        let mut builder = ManifestBuilder::new();
        let mut buckets: Vec<ManifestBucket> =
            vec![ManifestBucket::arbitrary(&mut unstructured).unwrap()];
        let mut proof_ids: Vec<ManifestProof> =
            vec![ManifestProof::arbitrary(&mut unstructured).unwrap()];

        let resource_address = unstructured
            .choose(&self.all_resource_addresses[..])
            .unwrap()
            .clone();
        let component_address = unstructured
            .choose(&self.component_addresses[..])
            .unwrap()
            .clone();
        let non_fungible_resource_address = unstructured
            .choose(&self.non_fungible_resource_addresses[..])
            .unwrap()
            .clone();

        let mut global_addresses = {
            let package_address = unstructured
                .choose(&self.package_addresses[..])
                .unwrap()
                .clone();
            vec![
                GlobalAddress::from(component_address),
                GlobalAddress::from(resource_address),
                GlobalAddress::from(package_address),
            ]
        };
        // TODO: if resource_address of not NonFungible resource is given then we got panic in get_mapped_substate
        // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: UnexpectedSize { expected: 2, actual: 1 }', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-stores/src/interface.rs:200:41
        let non_fungible_ids =
            self.get_non_fungible_local_id(component_address, non_fungible_resource_address);

        let public_key = unstructured.choose(&self.public_keys[..]).unwrap().clone();

        let fee = Decimal::arbitrary(&mut unstructured).unwrap();
        builder.lock_fee(component_address, fee);

        let mut i = 0;
        while i < INSTRUCTION_MAX_CNT && unstructured.len() > 0 {
            let next: u8 = unstructured
                .int_in_range(0..=ast::Instruction::COUNT as u8 - 1)
                .unwrap();
            let instruction = match next {
                // AssertWorktopContains
                0 => Some(Instruction::AssertWorktopContains { resource_address }),
                // AssertWorktopContainsByAmount
                1 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::AssertWorktopContainsByAmount {
                        amount,
                        resource_address,
                    })
                }
                // AssertWorktopContainsByIds
                2 => Some(Instruction::AssertWorktopContainsByIds {
                    ids: non_fungible_ids.clone(),
                    resource_address,
                }),
                // BurnResource
                3 => {
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();
                    Some(Instruction::BurnResource { bucket_id })
                }
                // CallFunction
                4 => {
                    // TODO
                    None
                }
                // CallMethod
                5 => {
                    // TODO
                    None
                }
                // ClaimComponentRoyalty
                6 => Some(Instruction::ClaimComponentRoyalty { component_address }),
                // ClaimPackageRoyalty
                7 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    Some(Instruction::ClaimPackageRoyalty { package_address })
                }
                // ClearAuthZone
                8 => Some(Instruction::ClearAuthZone),
                // ClearSignatureProofs
                9 => Some(Instruction::ClearSignatureProofs),
                // CloneProof
                10 => {
                    let proof_id = *unstructured.choose(&proof_ids[..]).unwrap();
                    Some(Instruction::CloneProof { proof_id })
                }
                // CreateAccessController
                11 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();
                    // TODO: crash when using arbitrary RuleSet
                    // - thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', src/fuzz_tx.rs:358:31
                    // - crash if any role equals
                    //   Protected(ProofRule(AmountOf(Static(9346522905005059338871465549192114877207573668753774438020.279842539602103386), Static(ResourceAddress(8fca9cf99eb51bbcbd774efea6def526efa9ac26eadfbc5c5f26b4fe4286))))), recovery_role: AllowAll, confirmation_role: DenyAll }
                    #[cfg(not(feature = "skip_crash"))]
                    let rule_set = RuleSet::arbitrary(&mut unstructured).unwrap();
                    #[cfg(feature = "skip_crash")]
                    let rule_set = RuleSet {
                        primary_role: AccessRule::AllowAll,
                        recovery_role: AccessRule::AllowAll,
                        confirmation_role: AccessRule::AllowAll,
                    };
                    let timed_recovery_delay_in_minutes =
                        <Option<u32>>::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
                        function_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
                        args: manifest_args!(bucket_id, rule_set, timed_recovery_delay_in_minutes),
                    })
                }
                // CreateAccount
                12 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                        function_name: ACCOUNT_CREATE_IDENT.to_string(),
                        args: to_manifest_value(
                            &AccountCreateInput::arbitrary(&mut unstructured).unwrap(),
                        ),
                    })
                }
                // CreateAccountAdvanced
                13 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    // TODO: crash when using arbitrary
                    // - thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // - AccountCreateAdvancedInput { config: AccessRulesConfig { direct_method_auth: {}, method_auth: {MethodKey { module_id: SELF, ident: "-!" }: AccessRule(DenyAll)}, grouped_auth: {"!LX": Protected(AllOf([AnyOf([]), AllOf([])])), "1)UZ": DenyAll, "t7": DenyAll}, default_auth: AccessRule(AllowAll), method_auth_mutability: {MethodKey { module_id: SELF, ident: "" }: AccessRule(Protected(AnyOf([AnyOf([AllOf([AllOf([]), ProofRule(AllOf(Static([StaticResource(ResourceAddress(dcd0c83141b9ff8080553b6190b5a7dc0cbde7854d9cf22b600480dcbc36)), Dynamic(SchemaPath([Field("\u{4}G<]y\u{5}\u{1f}")]))])))])]), AnyOf([ProofRule(AmountOf(Static(53647144799766708596252244031328084853448836832030149660791.749040275160793477), Static(ResourceAddress(d53666602d72af4e26da05ce175857e93a99a2ee5636a74734e7122ff9f2))))])])))}, grouped_auth_mutability: {"": DenyAll}, default_auth_mutability: AccessRule(DenyAll) } }
                    #[cfg(not(feature = "skip_crash"))]
                    let input = AccountCreateAdvancedInput::arbitrary(&mut unstructured).unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = AccountCreateAdvancedInput {
                        config: AccessRulesConfig::new()
                            .default(AccessRule::AllowAll, AccessRule::AllowAll),
                    };

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                        function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateFungibleResource
                14 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    // TODO: crash when using arbitrary
                    // - thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // - FungibleResourceManagerCreateInput { divisibility: 195, metadata: {"u\u{3}": ""}, access_rules: {UpdateNonFungibleData: (Protected(AnyOf([AllOf([ProofRule(AmountOf(Static(26154969881750291342967843398213318666330202269873300598763.295047154523616676), Static(ResourceAddress(6071567d817883153c3e5ad7505152a0cf52805d639e653336dca49f6c8d)))), ProofRule(Require(StaticNonFungible(ResourceAddress(4ba06972980bd34c26b0a3b4f9026d1fc145206120c7481aefff9bf9191c):#18136094832208207429#)))])])), DenyAll)} }
                    #[cfg(not(feature = "skip_crash"))]
                    let input =
                        FungibleResourceManagerCreateInput::arbitrary(&mut unstructured).unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = FungibleResourceManagerCreateInput {
                        divisibility: 18,
                        metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                        access_rules: BTreeMap::from([
                            (
                                ResourceMethodAuthKey::Withdraw,
                                (AccessRule::AllowAll, AccessRule::DenyAll),
                            ),
                            (
                                ResourceMethodAuthKey::Deposit,
                                (AccessRule::AllowAll, AccessRule::DenyAll),
                            ),
                        ]),
                    };

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateFungibleResourceWithInitialSupply
                15 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    // TODO: crash when using arbitrary
                    // - thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // - FungibleResourceManagerCreateWithInitialSupplyInput { divisibility: 220, metadata: {}, access_rules: {Burn: (DenyAll, DenyAll), Withdraw: (Protected(AnyOf([])), Protected(ProofRule(AmountOf(Dynamic(SchemaPath([])), Static(ResourceAddress(6c7bbb0abab0bb8a458c42209dbb23d885d698ece363e52f77dd4832efa8)))))), Recall: (AllowAll, AllowAll)}, initial_supply: -2148955441104578335242117384818719057515562139924293906511.547848591891882845 }
                    #[cfg(not(feature = "skip_crash"))]
                    let input = FungibleResourceManagerCreateWithInitialSupplyInput::arbitrary(
                        &mut unstructured,
                    )
                    .unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = FungibleResourceManagerCreateWithInitialSupplyInput {
                        divisibility: 18,
                        metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                        access_rules: BTreeMap::from([
                            (
                                ResourceMethodAuthKey::Withdraw,
                                (AccessRule::AllowAll, AccessRule::DenyAll),
                            ),
                            (
                                ResourceMethodAuthKey::Deposit,
                                (AccessRule::AllowAll, AccessRule::DenyAll),
                            ),
                        ]),
                        initial_supply: Decimal::arbitrary(&mut unstructured).unwrap(),
                    };

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                            .to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateIdentity
                16 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    let input = IdentityCreateInput::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: IDENTITY_BLUEPRINT.to_string(),
                        function_name: IDENTITY_CREATE_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateIdentityAdvanced
                17 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    // TODO: crash when using arbitrary
                    // - thread 'fuzz_tx::test_fuzz_tx' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // - IdentityCreateAdvancedInput { config: AccessRulesConfig { direct_method_auth: {MethodKey { module_id: SELF, ident: "" }: AccessRule(AllowAll)}, method_auth: {MethodKey { module_id: SELF, ident: "" }: Group(""), MethodKey { module_id: AccessRules, ident: "" }: AccessRule(AllowAll)}, grouped_auth: {"": Protected(AnyOf([]))}, default_auth: AccessRule(DenyAll), method_auth_mutability: {MethodKey { module_id: AccessRules, ident: "" }: Group("ǧv")}, grouped_auth_mutability: {"": Protected(AnyOf([ProofRule(CountOf(Dynamic(SchemaPath([Index(10162409116604676426)])), Dynamic(SchemaPath([Index(13698182042123810480)])))), ProofRule(Require(StaticNonFungible(ResourceAddress(20f6a7a71967b9349d46c1323bf8b4a369b5c376733a910f29d1fdef33ec):#14463890316188986874#)))])), "irRD4": DenyAll}, default_auth_mutability: AccessRule(AllowAll) } }
                    #[cfg(not(feature = "skip_crash"))]
                    let input = IdentityCreateAdvancedInput::arbitrary(&mut unstructured).unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = {
                        let owner_id =
                            NonFungibleGlobalId::from_public_key(&self.accounts[0].public_key);
                        let config = AccessRulesConfig::new()
                            .default(rule!(require(owner_id.clone())), rule!(require(owner_id)));
                        IdentityCreateAdvancedInput { config };
                    };

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: IDENTITY_BLUEPRINT.to_string(),
                        function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateNonFungibleResource
                18 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    // TODO: crash when using arbitrary
                    // - thread 'fuzz_tx::test_fuzz_tx' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // - NonFungibleResourceManagerCreateInput { id_type: Integer, non_fungible_schema: NonFungibleDataSchema { schema: Schema { type_kinds: [], type_metadata: [], type_validations: [] }, non_fungible: WellKnown(66), mutable_fields: {} }, metadata: {"": "", "%": "", "5": ""}, access_rules: {Burn: (Protected(AllOf([AllOf([AllOf([AllOf([]), AnyOf([ProofRule(AllOf(Static([StaticResource(ResourceAddress(6931f6c76e112721df1e958fb24ccf97497847a42429e23b27e22cae82b5))]))), AnyOf([]), ProofRule(AllOf(Dynamic(SchemaPath([])))), AllOf([ProofRule(CountOf(Dynamic(SchemaPath([])), Static([StaticNonFungible(ResourceAddress(5e3405cc2a8ff49842df0c42a192658f24f2380194fff3e1e8fe348952be):[])])))]), AllOf([])])])])])), AllowAll), UpdateMetadata: (AllowAll, DenyAll)} }
                    #[cfg(not(feature = "skip_crash"))]
                    let input = NonFungibleResourceManagerCreateInput::arbitrary(&mut unstructured)
                        .unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = NonFungibleResourceManagerCreateInput {
                        id_type: NonFungibleIdType::Integer,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                        metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                        access_rules: BTreeMap::from([
                            (
                                ResourceMethodAuthKey::Withdraw,
                                (AccessRule::AllowAll, AccessRule::DenyAll),
                            ),
                            (
                                ResourceMethodAuthKey::Deposit,
                                (AccessRule::AllowAll, AccessRule::DenyAll),
                            ),
                        ]),
                    };

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }

                // CreateNonFungibleResourceWithInitialSupply
                19 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    // TODO: crash when using arbitrary
                    // thread 'fuzz_tx::test_fuzz_tx' panicked at 'called `Result::unwrap()` on an `Err` value: MaxDepthExceeded(24)', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:45
                    // after increasing depth to 32
                    // thread 'fuzz_tx::test_fuzz_tx' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // NonFungibleResourceManagerCreateWithInitialSupplyManifestInput { id_type: Bytes, non_fungible_schema: NonFungibleDataSchema { schema: Schema { type_kinds: [], type_metadata: [], type_validations: [] }, non_fungible: WellKnown(66), mutable_fields: {} }, metadata: {"": "", "\u{1e}^ű": "", "<": "", ">%.": ""}, access_rules: {UpdateMetadata: (Protected(AnyOf([AllOf([AnyOf([]), AnyOf([AnyOf([AnyOf([AnyOf([]), AnyOf([ProofRule(AnyOf(Static([]))), AnyOf([])]), AllOf([AnyOf([AnyOf([]), AllOf([AllOf([AnyOf([AllOf([AnyOf([]), ProofRule(Require(StaticNonFungible(ResourceAddress(000000000000000000000000000000000000000000000000000000000000):<>)))])])])])])])])])])])])), AllowAll), Deposit: (Protected(ProofRule(CountOf(Static(234), Dynamic(SchemaPath([]))))), AllowAll)}, entries: {} }
                    #[cfg(not(feature = "skip_crash"))]
                    let input =
                        &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput::arbitrary(
                            &mut unstructured,
                        )
                        .unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = {
                        let mut entries = BTreeMap::new();
                        for _i in 0..u8::arbitrary(&mut unstructured).unwrap() {
                            let integer: u64 = unstructured.int_in_range(0..=1000).unwrap();
                            entries.insert(
                                NonFungibleLocalId::integer(integer),
                                (to_manifest_value(&(
                                    String::arbitrary(&mut unstructured).unwrap(),
                                    Decimal::arbitrary(&mut unstructured).unwrap(),
                                )),),
                            );
                        }
                        NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                            id_type: NonFungibleIdType::Integer,
                            non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                            metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                            access_rules: BTreeMap::from([
                                (
                                    ResourceMethodAuthKey::Withdraw,
                                    (AccessRule::AllowAll, AccessRule::DenyAll),
                                ),
                                (
                                    ResourceMethodAuthKey::Deposit,
                                    (AccessRule::AllowAll, AccessRule::DenyAll),
                                ),
                            ]),
                            entries,
                        }
                    };

                    Some(Instruction::CallFunction {
                        package_address,
                        blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name:
                            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                                .to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // CreateProofFromAuthZone
                20 => Some(Instruction::CreateProofFromAuthZone { resource_address }),
                // CreateProofFromAuthZoneByAmount
                21 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::CreateProofFromAuthZoneByAmount {
                        amount,
                        resource_address,
                    })
                }
                // CreateProofFromAuthZoneByIds
                22 => Some(Instruction::CreateProofFromAuthZoneByIds {
                    ids: non_fungible_ids.clone(),
                    resource_address,
                }),
                // CreateProofFromBucket
                23 => {
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();
                    Some(Instruction::CreateProofFromBucket { bucket_id })
                }
                // CreateValidator
                24 => {
                    let input = EpochManagerCreateValidatorInput { key: public_key };
                    Some(Instruction::CallMethod {
                        component_address,
                        method_name: EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
                        args: to_manifest_value(&input),
                    })
                }
                // DropAllProofs
                25 => Some(Instruction::DropAllProofs),
                // DropProof
                26 => {
                    let proof_id = *unstructured.choose(&proof_ids[..]).unwrap();
                    Some(Instruction::DropProof { proof_id })
                }
                // MintFungible
                27 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::MintFungible {
                        resource_address,
                        amount,
                    })
                }
                // MintNonFungible
                28 => {
                    // TODO: crash when using arbitrary
                    // - thread 'fuzz_tx::test_fuzz_tx' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // -  NonFungibleResourceManagerMintManifestInput { entries: {#13544282675100782418#: (I128 { value: -69359017762742067524339587899531658956 },), {0e66c9db-6a09-4c5a-760e-00d9e362d92c}: (U16 { value: 47995 },)} }
                    #[cfg(not(feature = "skip_crash"))]
                    let input =
                        NonFungibleResourceManagerMintManifestInput::arbitrary(&mut unstructured)
                            .unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = {
                        let mut entries = BTreeMap::new();
                        for _i in 0..u8::arbitrary(&mut unstructured).unwrap() {
                            let integer: u64 = unstructured.int_in_range(0..=1000).unwrap();
                            entries.insert(
                                NonFungibleLocalId::integer(integer),
                                (to_manifest_value(&(
                                    String::arbitrary(&mut unstructured).unwrap(),
                                    Decimal::arbitrary(&mut unstructured).unwrap(),
                                )),),
                            );
                        }
                        NonFungibleResourceManagerMintManifestInput { entries }
                    };

                    Some(Instruction::MintNonFungible {
                        resource_address,
                        args: to_manifest_value(&input),
                    })
                }
                // MintUuidNonFungible
                29 => {
                    // TODO: crash when using arbitrary
                    // - thread 'fuzz_tx::test_fuzz_tx' panicked at 'called `Result::unwrap()` on an `Err` value: UnknownValueKind(123)', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:45:55
                    // - NonFungibleResourceManagerMintUuidManifestInput { entries: [(I8 { value: 105 },), (I32 { value: 1989327177 },), (Array { element_value_kind: I32, elements: [I16 { value: 8591 }, Map { key_value_kind: Tuple, value_value_kind: I64, entries: [] }, Tuple { fields: [] }, Custom { value: NonFungibleLocalId(UUID(134904843443897012489077351069999533053)) }, I64 { value: 6355796430128099618 }, I32 { value: -1022678464 }] },), (I32 { value: -40799505 },)] }
                    #[cfg(not(feature = "skip_crash"))]
                    let input = NonFungibleResourceManagerMintUuidManifestInput::arbitrary(
                        &mut unstructured,
                    )
                    .unwrap();
                    #[cfg(feature = "skip_crash")]
                    let input = {
                        let mut entries = Vec::new();
                        for _i in 0..u8::arbitrary(&mut unstructured).unwrap() {
                            entries.push((to_manifest_value(&(
                                String::arbitrary(&mut unstructured).unwrap(),
                                Decimal::arbitrary(&mut unstructured).unwrap(),
                            )),));
                        }
                        NonFungibleResourceManagerMintUuidManifestInput { entries }
                    };

                    Some(Instruction::MintUuidNonFungible {
                        resource_address,
                        args: to_manifest_value(&input),
                    })
                }
                // PopFromAuthZone
                30 => Some(Instruction::PopFromAuthZone {}),
                // PublishPackage | PublishPackageAdvanced
                31 | 32 => {
                    // Publishing package involves a compilation by scrypto compiler.
                    // In case of AFL invoking external tool breaks fuzzing.
                    // For now we skip this step
                    // TODO: compile some packages before starting AFL and read compiled
                    //  binaries in AFL
                    None
                }
                // PushToAuthZone
                33 => {
                    let proof_id = *unstructured.choose(&proof_ids[..]).unwrap();
                    Some(Instruction::PushToAuthZone { proof_id })
                }
                // RecallResource
                34 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    // TODO: crash when using arbitrary
                    // - thread 'fuzz_tx::test_fuzz_tx' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine/src/blueprints/transaction_processor/tx_processor.rs:92:85
                    // - vault_id: Address(000000000000000000000000000000000000000000000000000000000000)
                    #[cfg(not(feature = "skip_crash"))]
                    // TODO: try to find some valid vault_ids and randomly choose it or generate
                    // if not found
                    let vault_id = Some(LocalAddress::arbitrary(&mut unstructured).unwrap());

                    #[cfg(feature = "skip_crash")]
                    let vault_id = {
                        let vaults = self
                            .runner
                            .get_component_vaults(component_address, resource_address);
                        if vaults.len() > 0 {
                            Some(*unstructured.choose(&vaults[..]).unwrap())
                        } else {
                            None
                        }
                    };

                    vault_id.map(|vault_id| Instruction::RecallResource {
                        vault_id: LocalAddress::new_unchecked(vault_id.into()),
                        amount,
                    })
                }
                // RemoveMetadata
                35 => {
                    global_addresses.push(
                        GlobalAddress::arbitrary(&mut unstructured).unwrap());
                    let entity_address = *unstructured.choose(&global_addresses[..]).unwrap();
                    let key = String::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::RemoveMetadata {
                        entity_address,
                        key,
                    })
                }
                // ReturnToWorktop
                36 => {
                    let bucket_id = *unstructured.choose(&buckets[..]).unwrap();
                    Some(Instruction::ReturnToWorktop { bucket_id })
                }
                // SetComponentRoyaltyConfig
                37 => {
                    let royalty_config = RoyaltyConfig::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::SetComponentRoyaltyConfig {
                        component_address,
                        royalty_config,
                    })
                }
                // SetMetadata
                38 => {
                    global_addresses.push(
                        GlobalAddress::arbitrary(&mut unstructured).unwrap());
                    let entity_address = *unstructured.choose(&global_addresses[..]).unwrap();
                    let key = String::arbitrary(&mut unstructured).unwrap();
                    // TODO: crash if using arbitrary on whole MetadataEntry
                    // - thread 'fuzz_tx::test_generate_fuzz_input_data' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine/src/transaction/reference_extractor.rs:77:90
                    // - SetMetadata { entity_address: Address(078e1d6def2b37e823e0d675a74ef0b790e8edab4d35db4c79cb40dd6f1a), key: "|\u{4}Ӕq?d,9", value: Value(Address(Address(a320d2a933e64e0461b08f89481dbcffe7007c90c1f188fbdcbb1c9c589d))) }
                    #[cfg(not(feature = "skip_crash"))]
                    let value = MetadataEntry::arbitrary(&mut unstructured).unwrap();
                    #[cfg(feature = "skip_crash")]
                    let value = MetadataEntry::Value(MetadataValue::Address(GlobalAddress::from(
                        component_address,
                    )));
                    Some(Instruction::SetMetadata {
                        entity_address,
                        key,
                        value,
                    })
                }
                // SetMethodAccessRule
                39 => {
                    global_addresses.push(
                        GlobalAddress::arbitrary(&mut unstructured).unwrap());
                    let entity_address = *unstructured.choose(&global_addresses[..]).unwrap();
                    let key = MethodKey::arbitrary(&mut unstructured).unwrap();
                    // TODO: crash if using arbitrary on AccessRule
                    // - thread 'fuzz_tx::test_generate_fuzz_input_data' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidCustomValue', /Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto/radix-engine/src/transaction/reference_extractor.rs:90:89
                    // - SetMethodAccessRule { entity_address: Address(020000000000000000000000000000000000000000000000000000000005), key: MethodKey { module_id: Royalty, ident: "" }, rule: Protected(ProofRule(AmountOf(Dynamic(SchemaPath([Index(11735663649062562571)])), Static(ResourceAddress(1f45ceaa693e21c0fc97c9b43ad41e9ab7ad99b77451aeeab8225cd71ccd))))) }
                    #[cfg(not(feature = "skip_crash"))]
                    let rule = AccessRule::arbitrary(&mut unstructured).unwrap();
                    #[cfg(feature = "skip_crash")]
                    let rule = AccessRule::AllowAll;
                    Some(Instruction::SetMethodAccessRule {
                        entity_address,
                        key,
                        rule,
                    })
                }
                // SetPackageRoyaltyConfig
                40 => {
                    self.package_addresses
                        .push(PackageAddress::arbitrary(&mut unstructured).unwrap());
                    let package_address = *unstructured.choose(&self.package_addresses[..]).unwrap();
                    let royalty_config = BTreeMap::<String, RoyaltyConfig>::arbitrary(&mut unstructured).unwrap();

                    Some(Instruction::SetPackageRoyaltyConfig {
                        package_address,
                        royalty_config,
                    })
                }
                // TakeFromWorktop
                41 => Some(Instruction::TakeFromWorktop { resource_address }),
                // TakeFromWorktopByAmount
                42 => {
                    let amount = Decimal::arbitrary(&mut unstructured).unwrap();
                    Some(Instruction::TakeFromWorktopByAmount {
                        amount,
                        resource_address,
                    })
                }
                // TakeFromWorktopByIds
                43 => Some(Instruction::TakeFromWorktopByIds {
                    ids: non_fungible_ids.clone(),
                    resource_address,
                }),
                _ => unreachable!(
                    "Not all instructions (current count is {}) covered by this match, current instruction {}",
                    ast::Instruction::COUNT, next
                ),
            };
            match instruction {
                Some(instruction) => {
                    let (_, bucket_id, proof_id) = builder.add_instruction(instruction);
                    match bucket_id {
                        Some(bucket_id) => buckets.push(bucket_id),
                        None => {}
                    }
                    match proof_id {
                        Some(proof_id) => proof_ids.push(proof_id),
                        None => {}
                    }
                    i += 1;
                }
                None => {}
            }
        }

        let manifest = builder.build();
        Ok(manifest)
    }

    pub fn fuzz_tx_manifest(&mut self, data: &[u8]) -> TxStatus {
        #[cfg(feature = "decode_tx_manifest")]
        let result = manifest_decode::<TransactionManifest>(data);
        #[cfg(not(feature = "decode_tx_manifest"))]
        let result = self.build_manifest(data);

        match result {
            #[allow(unused_mut)]
            Ok(mut manifest) => {
                #[cfg(feature = "smart_mutate")]
                self.smart_mutate_manifest(&mut manifest);

                let receipt = self.runner.execute_manifest(
                    manifest,
                    vec![NonFungibleGlobalId::from_public_key(
                        &self.accounts[0].public_key,
                    )],
                );
                if receipt.is_commit_success() {
                    TxStatus::CommitSuccess
                } else {
                    TxStatus::CommitFailure
                }
            }
            Err(_err) => TxStatus::DecodeError,
        }
    }
}

#[derive(Debug)]
pub enum TxStatus {
    // Transaction manifest build error
    #[allow(unused)]
    ManifestBuildError,
    // Transaction commit success
    CommitSuccess,
    // Transaction commit failure
    CommitFailure,
    // Transaction manifest parse error
    #[allow(unused)]
    DecodeError,
}

// This test tries is supposed to generate fuzz input data.
// It generates and executes manifest. If transaction successful then save the manifest data.
#[test]
#[cfg(not(feature = "decode_tx_manifest"))]
fn test_generate_fuzz_input_data() {
    use rand::{Rng, RngCore};
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(1234);
    let mut fuzzer = TxFuzzer::new();
    for _ in 0..5000 {
        let len = rng.gen_range(0..1024);
        let mut bytes: Vec<u8> = (0..len).map(|_| rng.gen_range(0..u8::MAX)).collect();
        rng.fill_bytes(&mut bytes[..]);

        fuzzer.reset_runner();
        match fuzzer.fuzz_tx_manifest(&bytes[..]) {
            TxStatus::CommitSuccess => {
                let m_hash = hash(&bytes);
                let path = format!("manifest_{:?}.raw", m_hash);
                std::fs::write(&path, bytes).unwrap();
                println!("manifest dumped to file {}", &path);
            }
            _ => {}
        }
    }
}

// This test is supposed to generate fuzz input data.
// It runs radix-engine-tests tests with "dump_manifest_to_file" flag,
// which writes each used transaction manifest to file.
#[test]
#[cfg(feature = "decode_tx_manifest")]
fn test_generate_fuzz_input_data() {
    /*
    cargo nextest run -p radix-engine-tests --release --features dump_manifest_to_file
    mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${raw_dir}
    */
    use std::fs;

    use std::io::{BufRead, BufReader};
    use std::process::Command;
    use std::process::Stdio;
    const WORK_DIR: &str = "/Users/lukaszrubaszewski/work/radixdlt/radixdlt-scrypto";
    const PACKAGE: &str = "radix-engine-tests";

    let mut child = Command::new("cargo")
        .current_dir(WORK_DIR)
        .stdin(Stdio::null())
        .arg("nextest")
        .arg("run")
        .arg("-p")
        .arg(PACKAGE)
        .arg("--release")
        .arg("--features")
        .arg("dump_manifest_to_file")
        .spawn()
        .expect("failed to execute process");

    if let Some(stdout) = &mut child.stdout {
        let lines = BufReader::new(stdout).lines().enumerate().take(10);
        for (_, line) in lines {
            println!("{:?}", line);
        }
    }

    child.wait().expect("failed to wait");

    let entries = fs::read_dir(format!("{}/{}", WORK_DIR, PACKAGE)).unwrap();

    entries
        .filter_map(|entry| Some(entry.unwrap()))
        .for_each(|entry| {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_str().unwrap();
            if fname.ends_with(".raw") && fname.starts_with("manifest_") {
                fs::rename(entry.path(), fname).unwrap();
            }
        });
}

// Initialize static objects outside the fuzzing loop to assure deterministic instrumentation
// output across runs.
pub fn fuzz_tx_init_statics() {
    // Following code initializes secp256k1::SECP256K1 global static context
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(100).unwrap();
    let _public_key = private_key.public_key();
}
