use std::convert::Infallible;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use radix_engine::engine::RuntimeError;
use radix_engine::engine::{KernelError, ModuleError, ScryptoInterpreter};
use radix_engine::ledger::*;
use radix_engine::model::{
    export_abi, export_abi_by_component, extract_abi, GlobalAddressSubstate, MetadataSubstate,
    ValidatorSetSubstate,
};
use radix_engine::state_manager::StagedSubstateStoreManager;
use radix_engine::transaction::{
    execute_and_commit_transaction, execute_preview, execute_transaction, ExecutionConfig,
    FeeReserveConfig, PreviewError, PreviewResult, TransactionReceipt,
};
use radix_engine::types::*;
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use radix_engine_constants::*;
use radix_engine_interface::api::types::{RENodeId, VaultOffset};
use radix_engine_interface::constants::EPOCH_MANAGER;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::{
    AccessRule, AccessRules, ClockInvocation, EpochManagerInvocation, FromPublicKey,
    NativeInvocation, NonFungibleAddress, NonFungibleIdTypeId,
};
use radix_engine_interface::modules::auth::AuthAddresses;
use radix_engine_interface::node::NetworkDefinition;
use radix_engine_interface::time::Instant;
use radix_engine_interface::{dec, rule};
use scrypto::component::Mutability;
use scrypto::component::Mutability::*;
use scrypto::NonFungibleData;
use transaction::builder::ManifestBuilder;
use transaction::model::{Executable, Instruction, SystemTransaction, TransactionManifest};
use transaction::model::{PreviewIntent, TestTransaction};
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::TestIntentHashManager;

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, BTreeMap<String, BlueprintAbi>) {
        // Build
        let status = Command::new("cargo")
            .current_dir(package_dir.as_ref())
            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
            .status()
            .unwrap();
        if !status.success() {
            panic!("Failed to compile package: {:?}", package_dir.as_ref());
        }

        // Find wasm path
        let mut cargo = package_dir.as_ref().to_owned();
        cargo.push("Cargo.toml");
        let wasm_name = if cargo.exists() {
            let content = fs::read_to_string(&cargo).expect("Failed to read the Cargo.toml file");
            Self::extract_crate_name(&content)
                .expect("Failed to extract crate name from the Cargo.toml file")
                .replace("-", "_")
        } else {
            // file name
            package_dir
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                .replace("-", "_")
        };
        let mut path = PathBuf::from_str(&get_cargo_target_directory(&cargo)).unwrap(); // Infallible;
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(wasm_name);
        path.set_extension("wasm");

        // Extract ABI
        let code = fs::read(&path).unwrap_or_else(|err| {
            panic!(
                "Failed to read built WASM from path {:?} - {:?}",
                &path, err
            )
        });
        let abi = extract_abi(&code).unwrap();

        (code, abi)
    }

    // Naive pattern matching to find the crate name.
    fn extract_crate_name(mut content: &str) -> Result<String, ()> {
        let idx = content.find("name").ok_or(())?;
        content = &content[idx + 4..];

        let idx = content.find('"').ok_or(())?;
        content = &content[idx + 1..];

        let end = content.find('"').ok_or(())?;
        Ok(content[..end].to_string())
    }
}

pub struct TestRunner {
    scrypto_interpreter: ScryptoInterpreter<DefaultWasmEngine>,
    staged_substate_store_manager: StagedSubstateStoreManager<TypedInMemorySubstateStore>,
    intent_hash_manager: TestIntentHashManager,
    next_private_key: u64,
    next_transaction_nonce: u64,
    trace: bool,
}

impl TestRunner {
    pub fn new(trace: bool) -> Self {
        Self::new_with_genesis(trace, create_genesis(BTreeMap::new(), 1u64, 1u64))
    }

    pub fn new_with_genesis(trace: bool, genesis: SystemTransaction) -> Self {
        let scrypto_interpreter = ScryptoInterpreter {
            wasm_metering_config: WasmMeteringConfig::V0,
            wasm_engine: DefaultWasmEngine::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
        };
        let mut substate_store = TypedInMemorySubstateStore::new();
        let transaction_receipt = execute_transaction(
            &mut substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &genesis.get_executable(vec![AuthAddresses::system_role()]),
        );
        let commit_result = transaction_receipt.expect_commit();
        commit_result.outcome.expect_success();
        commit_result.state_updates.commit(&mut substate_store);
        Self {
            scrypto_interpreter,
            staged_substate_store_manager: StagedSubstateStoreManager::new(substate_store),
            intent_hash_manager: TestIntentHashManager::new(),
            next_private_key: 1, // 0 is invalid
            next_transaction_nonce: 0,
            trace,
        }
    }

    pub fn substate_store(&self) -> &TypedInMemorySubstateStore {
        &self.staged_substate_store_manager.root
    }

    pub fn substate_store_mut(&mut self) -> &mut TypedInMemorySubstateStore {
        &mut self.staged_substate_store_manager.root
    }

    pub fn next_private_key(&mut self) -> u64 {
        self.next_private_key += 1;
        self.next_private_key - 1
    }

    pub fn next_transaction_nonce(&mut self) -> u64 {
        self.next_transaction_nonce += 1;
        self.next_transaction_nonce - 1
    }

    pub fn new_key_pair(&mut self) -> (EcdsaSecp256k1PublicKey, EcdsaSecp256k1PrivateKey) {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(self.next_private_key()).unwrap();
        let public_key = private_key.public_key();

        (public_key, private_key)
    }

    pub fn new_key_pair_with_auth_address(
        &mut self,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        NonFungibleAddress,
    ) {
        let key_pair = self.new_allocated_account();
        (
            key_pair.0,
            key_pair.1,
            NonFungibleAddress::from_public_key(&key_pair.0),
        )
    }

    pub fn get_metadata(&mut self, address: GlobalAddress) -> BTreeMap<String, String> {
        let node_id = RENodeId::Global(address);
        let global = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                node_id,
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|s| s.substate.to_runtime())
            .unwrap();

        let underlying_node = global.global().node_deref();

        let metadata = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                underlying_node,
                SubstateOffset::Metadata(MetadataOffset::Metadata),
            ))
            .map(|s| s.substate.to_runtime())
            .unwrap();

        let metadata: MetadataSubstate = metadata.into();
        metadata.metadata
    }

    pub fn deref_component(&mut self, component_address: ComponentAddress) -> Option<RENodeId> {
        let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
        let global = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                node_id,
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|s| s.substate.to_runtime())?;
        Some(global.global().node_deref())
    }

    pub fn deref_package(&mut self, package_address: PackageAddress) -> Option<RENodeId> {
        let node_id = RENodeId::Global(GlobalAddress::Package(package_address));
        let global = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                node_id,
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|s| s.substate.to_runtime())?;
        Some(global.global().node_deref())
    }

    pub fn inspect_component_royalty(
        &mut self,
        component_address: ComponentAddress,
    ) -> Option<Decimal> {
        let node_id = self.deref_component(component_address)?;

        if let Some(output) = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                node_id,
                SubstateOffset::Component(ComponentOffset::RoyaltyAccumulator),
            ))
        {
            let royalty_vault: Own = output
                .substate
                .component_royalty_accumulator()
                .royalty
                .clone();

            self.staged_substate_store_manager
                .root
                .get_substate(&SubstateId(
                    RENodeId::Vault(royalty_vault.vault_id()),
                    SubstateOffset::Vault(VaultOffset::Vault),
                ))
                .map(|output| output.substate.vault().0.amount())
        } else {
            None
        }
    }

    pub fn inspect_package_royalty(&mut self, package_address: PackageAddress) -> Option<Decimal> {
        let node_id = self.deref_package(package_address)?;

        if let Some(output) = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                node_id,
                SubstateOffset::Package(PackageOffset::RoyaltyAccumulator),
            ))
        {
            let royalty_vault: Own = output
                .substate
                .package_royalty_accumulator()
                .royalty
                .clone();

            self.staged_substate_store_manager
                .root
                .get_substate(&SubstateId(
                    RENodeId::Vault(royalty_vault.vault_id()),
                    SubstateOffset::Vault(VaultOffset::Vault),
                ))
                .map(|output| output.substate.vault().0.amount())
        } else {
            None
        }
    }

    pub fn get_component_vaults(
        &mut self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Vec<VaultId> {
        let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
        let mut vault_finder = VaultFinder::new(resource_address);

        let mut state_tree_visitor = StateTreeTraverser::new(
            &self.staged_substate_store_manager.root,
            &mut vault_finder,
            100,
        );
        state_tree_visitor
            .traverse_all_descendents(None, node_id)
            .unwrap();
        vault_finder.to_vaults()
    }

    pub fn inspect_nft_vault(&mut self, vault_id: VaultId) -> Option<BTreeSet<NonFungibleId>> {
        self.substate_store()
            .get_substate(&SubstateId(
                RENodeId::Vault(vault_id),
                SubstateOffset::Vault(VaultOffset::Vault),
            ))
            .map(|output| output.substate.vault().0.ids().clone())
    }

    pub fn get_component_resources(
        &mut self,
        component_address: ComponentAddress,
    ) -> HashMap<ResourceAddress, Decimal> {
        let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
        let mut accounter = ResourceAccounter::new(&self.staged_substate_store_manager.root);
        accounter.add_resources(node_id).unwrap();
        accounter.into_map()
    }

    pub fn load_account_from_faucet(&mut self, account_address: ComponentAddress) {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .call_method(FAUCET_COMPONENT, "free", args!())
            .take_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(account_address, "deposit", args!(bucket))
            })
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }

    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &AccessRule) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .call_method(FAUCET_COMPONENT, "free", args!())
            .take_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.new_account_with_resource(withdraw_auth, bucket)
            })
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();

        receipt
            .expect_commit()
            .entity_changes
            .new_component_addresses[0]
    }

    pub fn new_virtual_account(
        &mut self,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        ComponentAddress,
    ) {
        let (pub_key, priv_key) = self.new_key_pair();
        let account = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(pub_key.clone()),
        );
        self.load_account_from_faucet(account);
        (pub_key, priv_key, account)
    }

    pub fn deref_component_address(&mut self, component_address: ComponentAddress) -> RENodeId {
        let substate: GlobalAddressSubstate = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|output| output.substate.to_runtime().into())
            .unwrap();

        substate.node_deref()
    }

    pub fn deref_package_address(&mut self, package_address: PackageAddress) -> RENodeId {
        let substate: GlobalAddressSubstate = self
            .staged_substate_store_manager
            .root
            .get_substate(&SubstateId(
                RENodeId::Global(GlobalAddress::Package(package_address)),
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|output| output.substate.to_runtime().into())
            .unwrap();

        substate.node_deref()
    }

    pub fn deref_system_address(&mut self, system_address: SystemAddress) -> RENodeId {
        let substate: GlobalAddressSubstate = self
            .substate_store()
            .get_substate(&SubstateId(
                RENodeId::Global(GlobalAddress::System(system_address)),
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|output| output.substate.to_runtime().into())
            .unwrap();

        substate.node_deref()
    }

    pub fn get_validator_with_key(&mut self, key: &EcdsaSecp256k1PublicKey) -> SystemAddress {
        let node_id = self.deref_system_address(EPOCH_MANAGER);
        let substate_id = SubstateId(
            node_id,
            SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet),
        );
        let substate: ValidatorSetSubstate = self
            .substate_store()
            .get_substate(&substate_id)
            .unwrap()
            .substate
            .to_runtime()
            .into();
        substate
            .validator_set
            .iter()
            .find(|(_, v)| v.key.eq(key))
            .unwrap()
            .0
            .clone()
    }

    pub fn new_allocated_account(
        &mut self,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        ComponentAddress,
    ) {
        let key_pair = self.new_key_pair();
        let withdraw_auth = rule!(require(NonFungibleAddress::from_public_key(&key_pair.0)));
        let account = self.new_account_with_auth_rule(&withdraw_auth);
        (key_pair.0, key_pair.1, account)
    }

    pub fn new_account(
        &mut self,
        is_virtual: bool,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        ComponentAddress,
    ) {
        if is_virtual {
            self.new_virtual_account()
        } else {
            self.new_allocated_account()
        }
    }

    pub fn new_validator(&mut self) -> (EcdsaSecp256k1PublicKey, SystemAddress) {
        let (pub_key, _) = self.new_key_pair();
        let address = self.new_validator_with_pub_key(pub_key);
        (pub_key, address)
    }

    pub fn new_validator_with_pub_key(
        &mut self,
        pub_key: EcdsaSecp256k1PublicKey,
    ) -> SystemAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 10.into())
            .create_validator(pub_key)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        receipt.expect_commit().entity_changes.new_system_addresses[0]
    }

    pub fn publish_package(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRules,
    ) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .publish_package(code, abi, royalty_config, metadata, access_rules)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        receipt.expect_commit().entity_changes.new_package_addresses[0]
    }

    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        owner_badge: NonFungibleAddress,
    ) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .publish_package_with_owner(code, abi, owner_badge)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        receipt.expect_commit().entity_changes.new_package_addresses[0]
    }

    pub fn compile_and_publish<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        let (code, abi) = Compile::compile(package_dir);
        self.publish_package(
            code,
            abi,
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRules::new(),
        )
    }

    pub fn compile_and_publish_with_owner<P: AsRef<Path>>(
        &mut self,
        package_dir: P,
        owner_badge: NonFungibleAddress,
    ) -> PackageAddress {
        let (code, abi) = Compile::compile(package_dir);
        self.publish_package_with_owner(code, abi, owner_badge)
    }

    pub fn execute_manifest_ignoring_fee(
        &mut self,
        mut manifest: TransactionManifest,
        initial_proofs: Vec<NonFungibleAddress>,
    ) -> TransactionReceipt {
        manifest.instructions.insert(
            0,
            transaction::model::BasicInstruction::CallMethod {
                component_address: FAUCET_COMPONENT,
                method_name: "lock_fee".to_string(),
                args: args!(dec!("100")),
            },
        );
        self.execute_manifest(manifest, initial_proofs)
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifest,
        initial_proofs: Vec<NonFungibleAddress>,
    ) -> TransactionReceipt {
        self.execute_manifest_with_cost_unit_limit(
            manifest,
            initial_proofs,
            DEFAULT_COST_UNIT_LIMIT,
        )
    }

    pub fn execute_manifest_with_cost_unit_limit(
        &mut self,
        manifest: TransactionManifest,
        initial_proofs: Vec<NonFungibleAddress>,
        cost_unit_limit: u32,
    ) -> TransactionReceipt {
        let transactions =
            TestTransaction::new(manifest, self.next_transaction_nonce(), cost_unit_limit);
        let executable = transactions.get_executable(initial_proofs);

        let fee_reserve_config = FeeReserveConfig::default();
        let mut execution_config = ExecutionConfig::default();
        execution_config.trace = self.trace;

        self.execute_transaction_with_config(executable, &fee_reserve_config, &execution_config)
    }

    pub fn execute_transaction(&mut self, executable: Executable) -> TransactionReceipt {
        let fee_config = FeeReserveConfig::default();
        let mut execution_config = ExecutionConfig::default();
        execution_config.trace = self.trace;

        self.execute_transaction_with_config(executable, &fee_config, &execution_config)
    }

    pub fn execute_transaction_with_config(
        &mut self,
        executable: Executable,
        fee_reserve_config: &FeeReserveConfig,
        execution_config: &ExecutionConfig,
    ) -> TransactionReceipt {
        execute_and_commit_transaction(
            &mut self.staged_substate_store_manager.root,
            &self.scrypto_interpreter,
            fee_reserve_config,
            execution_config,
            &executable,
        )
    }

    pub fn preview(
        &mut self,
        preview_intent: PreviewIntent,
        network: &NetworkDefinition,
    ) -> Result<PreviewResult, PreviewError> {
        execute_preview(
            &self.staged_substate_store_manager.root,
            &mut self.scrypto_interpreter,
            &self.intent_hash_manager,
            network,
            preview_intent,
        )
    }

    pub fn export_abi(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> BlueprintAbi {
        export_abi(
            &self.staged_substate_store_manager.root,
            package_address,
            blueprint_name,
        )
        .expect("Failed to export ABI")
    }

    pub fn export_abi_by_component(&mut self, component_address: ComponentAddress) -> BlueprintAbi {
        export_abi_by_component(&self.staged_substate_store_manager.root, component_address)
            .expect("Failed to export ABI")
    }

    pub fn lock_resource_auth(
        &mut self,
        function: &str,
        auth: ResourceAddress,
        token: ResourceAddress,
        account: ComponentAddress,
        signer_public_key: EcdsaSecp256k1PublicKey,
    ) {
        let package = self.compile_and_publish("./tests/blueprints/resource_creator");
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .create_proof_from_account(account, auth)
            .call_function(package, "ResourceCreator", function, args!(token))
            .build();
        self.execute_manifest(
            manifest,
            vec![NonFungibleAddress::from_public_key(&signer_public_key)],
        )
        .expect_commit_success();
    }

    pub fn update_resource_auth(
        &mut self,
        function: &str,
        auth: ResourceAddress,
        token: ResourceAddress,
        set_auth: ResourceAddress,
        account: ComponentAddress,
        signer_public_key: EcdsaSecp256k1PublicKey,
    ) {
        let package = self.compile_and_publish("./tests/blueprints/resource_creator");
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .create_proof_from_account(account, auth)
            .call_function(package, "ResourceCreator", function, args!(token, set_auth))
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build();
        self.execute_manifest(
            manifest,
            vec![NonFungibleAddress::from_public_key(&signer_public_key)],
        )
        .expect_commit_success();
    }

    fn create_fungible_resource_and_deposit(
        &mut self,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        to: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .create_fungible_resource(0, BTreeMap::new(), access_rules, Some(5.into()))
            .call_method(
                to,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        receipt
            .expect_commit()
            .entity_changes
            .new_resource_addresses[0]
    }

    pub fn create_restricted_token(
        &mut self,
        account: ComponentAddress,
    ) -> (
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
    ) {
        let mint_auth = self.create_non_fungible_resource(account);
        let burn_auth = self.create_non_fungible_resource(account);
        let withdraw_auth = self.create_non_fungible_resource(account);
        let recall_auth = self.create_non_fungible_resource(account);
        let update_metadata_auth = self.create_non_fungible_resource(account);
        let admin_auth = self.create_non_fungible_resource(account);

        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            Mint,
            (
                rule!(require(mint_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Burn,
            (
                rule!(require(burn_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Withdraw,
            (
                rule!(require(withdraw_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Recall,
            (
                rule!(require(recall_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            UpdateMetadata,
            (
                rule!(require(update_metadata_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Deposit,
            (rule!(allow_all), MUTABLE(rule!(require(admin_auth)))),
        );

        let token_address = self.create_fungible_resource_and_deposit(access_rules, account);

        (
            token_address,
            mint_auth,
            burn_auth,
            withdraw_auth,
            recall_auth,
            update_metadata_auth,
            admin_auth,
        )
    }

    pub fn create_recallable_token(&mut self, account: ComponentAddress) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Recall, (rule!(allow_all), LOCKED));

        self.create_fungible_resource_and_deposit(access_rules, account)
    }

    pub fn create_restricted_burn_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(Burn, (rule!(require(auth_resource_address)), LOCKED));
        let resource_address = self.create_fungible_resource_and_deposit(access_rules, account);

        (auth_resource_address, resource_address)
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(require(auth_resource_address)), LOCKED),
        );
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        let resource_address = self.create_fungible_resource_and_deposit(access_rules, account);

        (auth_resource_address, resource_address)
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));

        let mut entries = BTreeMap::new();
        entries.insert(NonFungibleId::Number(1), SampleNonFungibleData {});
        entries.insert(NonFungibleId::Number(2), SampleNonFungibleData {});
        entries.insert(NonFungibleId::Number(3), SampleNonFungibleData {});

        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .create_non_fungible_resource(
                NonFungibleIdTypeId::Number,
                BTreeMap::new(),
                access_rules,
                Some(entries),
            )
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        receipt
            .expect_commit()
            .entity_changes
            .new_resource_addresses[0]
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .create_fungible_resource(divisibility, BTreeMap::new(), access_rules, Some(amount))
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        receipt
            .expect_commit()
            .entity_changes
            .new_resource_addresses[0]
    }

    pub fn create_mintable_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Mint, (rule!(allow_all), LOCKED));
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .create_fungible_resource(divisibility, BTreeMap::new(), access_rules, Some(amount))
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        receipt
            .expect_commit()
            .entity_changes
            .new_resource_addresses[0]
    }

    pub fn instantiate_component<F>(
        &mut self,
        initial_proofs: Vec<NonFungibleAddress>,
        handler: F,
    ) -> ComponentAddress
    where
        F: FnOnce(&mut ManifestBuilder) -> &mut ManifestBuilder,
    {
        let manifest = ManifestBuilder::new()
            .call_method(FAUCET_COMPONENT, "lock_fee", args!(dec!("10")))
            .borrow_mut(|builder| Result::<_, Infallible>::Ok(handler(builder)))
            .unwrap()
            .build();

        let receipt = self.execute_manifest(manifest, initial_proofs);
        receipt.new_component_addresses()[0]
    }

    pub fn set_current_epoch(&mut self, epoch: u64) {
        let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
            EpochManagerInvocation::SetEpoch(EpochManagerSetEpochInvocation {
                receiver: EPOCH_MANAGER,
                epoch,
            }),
        ))];
        let blobs = vec![];
        let nonce = self.next_transaction_nonce();

        let receipt = self.execute_transaction(
            SystemTransaction {
                instructions,
                blobs,
                nonce,
            }
            .get_executable(vec![AuthAddresses::system_role()]),
        );
        receipt.expect_commit_success();
    }

    pub fn get_current_epoch(&mut self) -> u64 {
        let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
            EpochManagerInvocation::GetCurrentEpoch(EpochManagerGetCurrentEpochInvocation {
                receiver: EPOCH_MANAGER,
            }),
        ))];
        let blobs = vec![];
        let nonce = self.next_transaction_nonce();

        let receipt = self.execute_transaction(
            SystemTransaction {
                instructions,
                blobs,
                nonce,
            }
            .get_executable(vec![AuthAddresses::validator_role()]),
        );
        receipt.output(0)
    }

    pub fn set_current_time(&mut self, current_time_ms: i64) {
        let instructions = vec![Instruction::System(NativeInvocation::Clock(
            ClockInvocation::SetCurrentTime(ClockSetCurrentTimeInvocation {
                current_time_ms,
                receiver: CLOCK,
            }),
        ))];
        let blobs = vec![];
        let nonce = self.next_transaction_nonce();

        let receipt = self.execute_transaction(
            SystemTransaction {
                instructions,
                blobs,
                nonce,
            }
            .get_executable(vec![AuthAddresses::validator_role()]),
        );
        receipt.output(0)
    }

    pub fn get_current_time(&mut self, precision: TimePrecision) -> Instant {
        let instructions = vec![Instruction::System(NativeInvocation::Clock(
            ClockInvocation::GetCurrentTime(ClockGetCurrentTimeInvocation {
                precision,
                receiver: CLOCK,
            }),
        ))];
        let blobs = vec![];
        let nonce = self.next_transaction_nonce();

        let receipt = self.execute_transaction(
            SystemTransaction {
                instructions,
                blobs,
                nonce,
            }
            .get_executable(vec![AuthAddresses::validator_role()]),
        );
        receipt.output(0)
    }
}

pub fn is_auth_error(e: &RuntimeError) -> bool {
    matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(_)))
}

pub fn is_costing_error(e: &RuntimeError) -> bool {
    matches!(e, RuntimeError::ModuleError(ModuleError::CostingError(_)))
}

pub fn is_wasm_error(e: &RuntimeError) -> bool {
    matches!(e, RuntimeError::KernelError(KernelError::WasmError(..)))
}

pub fn wat2wasm(wat: &str) -> Vec<u8> {
    wabt::wat2wasm(
        wat.replace("${memcpy}", include_str!("snippets/memcpy.wat"))
            .replace("${memmove}", include_str!("snippets/memmove.wat"))
            .replace("${memset}", include_str!("snippets/memset.wat"))
            .replace("${buffer}", include_str!("snippets/buffer.wat")),
    )
    .expect("Failed to compiled WAT into WASM")
}

/// Gets the default cargo directory for the given crate.
/// This respects whether the crate is in a workspace.
pub fn get_cargo_target_directory(manifest_path: impl AsRef<OsStr>) -> String {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()
        .expect("Failed to call cargo metadata");
    if output.status.success() {
        let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .expect("Failed to parse cargo metadata");
        let target_directory = parsed
            .as_object()
            .and_then(|o| o.get("target_directory"))
            .and_then(|o| o.as_str())
            .expect("Failed to parse target_directory from cargo metadata");
        target_directory.to_owned()
    } else {
        panic!("Cargo metadata call was not successful");
    }
}

pub fn generate_single_function_abi(
    blueprint_name: &str,
    function_name: &str,
    output_type: Type,
) -> BTreeMap<String, BlueprintAbi> {
    let mut blueprint_abis = BTreeMap::new();
    blueprint_abis.insert(
        blueprint_name.to_string(),
        BlueprintAbi {
            structure: Type::Tuple {
                element_types: vec![],
            },
            fns: vec![Fn {
                ident: function_name.to_string(),
                mutability: Option::None,
                input: Type::Struct {
                    name: "Any".to_string(),
                    fields: Fields::Named { named: vec![] },
                },
                output: output_type,
                export_name: format!("{}_{}", blueprint_name, function_name),
            }],
        },
    );
    blueprint_abis
}

#[derive(NonFungibleData)]
struct SampleNonFungibleData {}
