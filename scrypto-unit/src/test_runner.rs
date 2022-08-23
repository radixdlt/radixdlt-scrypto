use radix_engine::constants::{
    DEFAULT_COST_UNIT_PRICE, DEFAULT_MAX_CALL_DEPTH, DEFAULT_SYSTEM_LOAN,
};
use radix_engine::engine::{ExecutionTrace, Kernel, ModuleError, SystemApi};
use radix_engine::engine::{RuntimeError, Track};
use radix_engine::fee::SystemLoanFeeReserve;
use radix_engine::ledger::*;
use radix_engine::model::{export_abi, export_abi_by_component, extract_package};
use radix_engine::state_manager::StagedSubstateStoreManager;
use radix_engine::transaction::{
    ExecutionConfig, PreviewError, PreviewExecutor, PreviewResult, TransactionExecutor,
    TransactionReceipt,
};
use radix_engine::wasm::{
    DefaultWasmEngine, DefaultWasmInstance, InstructionCostRules, WasmInstrumenter,
    WasmMeteringParams,
};
use sbor::describe::Fields;
use sbor::Type;
use scrypto::abi::{BlueprintAbi, Fn};
use scrypto::core::Network;
use scrypto::engine::types::{KeyValueStoreId, RENodeId, SubstateId, VaultId};
use scrypto::prelude::*;
use scrypto::prelude::{HashMap, Package};
use scrypto::values::ScryptoValue;
use scrypto::{abi, args};
use transaction::builder::ManifestBuilder;
use transaction::model::{ExecutableTransaction, TransactionManifest};
use transaction::model::{PreviewIntent, TestTransaction};
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::TestIntentHashManager;

pub struct TestRunner<'s, S: ReadableSubstateStore + WriteableSubstateStore> {
    execution_stores: StagedSubstateStoreManager<'s, S>,
    wasm_engine: DefaultWasmEngine,
    wasm_instrumenter: WasmInstrumenter,
    intent_hash_manager: TestIntentHashManager,
    next_private_key: u64,
    next_transaction_nonce: u64,
    trace: bool,
}

impl<'s, S: ReadableSubstateStore + WriteableSubstateStore> TestRunner<'s, S> {
    pub fn new(trace: bool, substate_store: &'s mut S) -> Self {
        Self {
            execution_stores: StagedSubstateStoreManager::new(substate_store),
            wasm_engine: DefaultWasmEngine::new(),
            wasm_instrumenter: WasmInstrumenter::new(),
            intent_hash_manager: TestIntentHashManager::new(),
            next_private_key: 1, // 0 is invalid
            next_transaction_nonce: 0,
            trace,
        }
    }

    pub fn next_transaction_nonce(&self) -> u64 {
        self.next_transaction_nonce
    }

    pub fn new_key_pair(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey) {
        let private_key = EcdsaPrivateKey::from_u64(self.next_private_key).unwrap();
        let public_key = private_key.public_key();

        self.next_private_key += 1;
        (public_key, private_key)
    }

    pub fn new_key_pair_with_auth_address(
        &mut self,
    ) -> (EcdsaPublicKey, EcdsaPrivateKey, NonFungibleAddress) {
        let key_pair = self.new_account();
        (
            key_pair.0,
            key_pair.1,
            NonFungibleAddress::from_public_key(&key_pair.0),
        )
    }

    pub fn inspect_component(
        &mut self,
        component_address: ComponentAddress,
    ) -> Option<radix_engine::model::ComponentInfo> {
        self.execution_stores
            .get_output_store(0)
            .get_substate(&SubstateId::ComponentInfo(component_address))
            .map(|output| output.substate.into())
    }

    pub fn inspect_component_state(
        &mut self,
        component_address: ComponentAddress,
    ) -> Option<radix_engine::model::ComponentState> {
        self.execution_stores
            .get_output_store(0)
            .get_substate(&SubstateId::ComponentState(component_address))
            .map(|output| output.substate.into())
    }

    pub fn inspect_key_value_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: Vec<u8>,
    ) -> Option<radix_engine::model::KeyValueStoreEntryWrapper> {
        self.execution_stores
            .get_output_store(0)
            .get_substate(&SubstateId::KeyValueStoreEntry(kv_store_id, key))
            .map(|output| output.substate.into())
    }

    pub fn inspect_vault(&mut self, vault_id: VaultId) -> Option<radix_engine::model::Vault> {
        self.execution_stores
            .get_output_store(0)
            .get_substate(&SubstateId::Vault(vault_id))
            .map(|output| output.substate.into())
    }

    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &AccessRule) -> ComponentAddress {
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_method(SYS_FAUCET_COMPONENT, "free_xrd", args!())
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.new_account_with_resource(withdraw_auth, bucket_id)
            })
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_success();

        receipt.new_component_addresses[0]
    }

    pub fn new_account(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) {
        let key_pair = self.new_key_pair();
        let withdraw_auth = rule!(require(NonFungibleAddress::from_public_key(&key_pair.0)));
        let account = self.new_account_with_auth_rule(&withdraw_auth);
        (key_pair.0, key_pair.1, account)
    }

    pub fn publish_package(&mut self, package: Package) -> PackageAddress {
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .publish_package(package)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_success();
        receipt.new_package_addresses[0]
    }

    pub fn extract_and_publish_package(&mut self, name: &str) -> PackageAddress {
        self.publish_package_with_code(compile_package!(format!("./tests/{}", name)))
    }

    pub fn publish_package_with_code(&mut self, code: Vec<u8>) -> PackageAddress {
        self.publish_package(extract_package(code).expect("Failed to extract package"))
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifest,
        signer_public_keys: Vec<EcdsaPublicKey>,
    ) -> TransactionReceipt {
        let mut receipts = self.execute_batch(vec![(manifest, signer_public_keys)]);
        receipts.pop().unwrap()
    }

    pub fn execute_manifest_ignoring_fee(
        &mut self,
        mut manifest: TransactionManifest,
        signer_public_keys: Vec<EcdsaPublicKey>,
    ) -> TransactionReceipt {
        manifest.instructions.insert(
            0,
            transaction::model::Instruction::CallMethod {
                component_address: SYS_FAUCET_COMPONENT,
                method_name: "lock_fee".to_string(),
                args: args!(dec!("1000")),
            },
        );
        self.execute_manifest(manifest, signer_public_keys)
    }

    pub fn execute_transaction<T: ExecutableTransaction>(
        &mut self,
        transaction: &T,
        params: &ExecutionConfig,
    ) -> TransactionReceipt {
        let node_id = self.create_child_node(0);
        let substate_store = &mut self.execution_stores.get_output_store(node_id);

        TransactionExecutor::new(
            substate_store,
            &mut self.wasm_engine,
            &mut self.wasm_instrumenter,
        )
        .execute(transaction, params)
    }

    pub fn execute_preview(
        &mut self,
        preview_intent: PreviewIntent,
    ) -> Result<PreviewResult, PreviewError> {
        let node_id = self.create_child_node(0);
        let substate_store = &mut self.execution_stores.get_output_store(node_id);

        PreviewExecutor::new(
            substate_store,
            &mut self.wasm_engine,
            &mut self.wasm_instrumenter,
            &self.intent_hash_manager,
        )
        .execute(preview_intent)
    }

    pub fn execute_batch(
        &mut self,
        manifests: Vec<(TransactionManifest, Vec<EcdsaPublicKey>)>,
    ) -> Vec<TransactionReceipt> {
        let node_id = self.create_child_node(0);
        let receipts = self.execute_batch_on_node(node_id, manifests);
        self.merge_node(node_id);
        receipts
    }

    pub fn create_child_node(&mut self, parent_id: u64) -> u64 {
        self.execution_stores.new_child_node(parent_id)
    }

    pub fn execute_batch_on_node(
        &mut self,
        node_id: u64,
        manifests: Vec<(TransactionManifest, Vec<EcdsaPublicKey>)>,
    ) -> Vec<TransactionReceipt> {
        let mut store = self.execution_stores.get_output_store(node_id);
        let mut receipts = Vec::new();
        for (manifest, signer_public_keys) in manifests {
            let transaction =
                TestTransaction::new(manifest, self.next_transaction_nonce, signer_public_keys);
            self.next_transaction_nonce += 1;
            let receipt = TransactionExecutor::new(
                &mut store,
                &mut self.wasm_engine,
                &mut self.wasm_instrumenter,
            )
            .execute_and_commit(
                &transaction,
                &ExecutionConfig {
                    cost_unit_price: DEFAULT_COST_UNIT_PRICE.parse().unwrap(),
                    max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                    system_loan: DEFAULT_SYSTEM_LOAN,
                    is_system: false,
                    trace: self.trace,
                },
            );
            receipts.push(receipt);
        }

        receipts
    }

    pub fn merge_node(&mut self, node_id: u64) {
        self.execution_stores.merge_to_parent(node_id);
    }

    pub fn export_abi(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> abi::BlueprintAbi {
        let output_store = self.execution_stores.get_output_store(0);
        export_abi(&output_store, package_address, blueprint_name).expect("Failed to export ABI")
    }

    pub fn export_abi_by_component(
        &mut self,
        component_address: ComponentAddress,
    ) -> abi::BlueprintAbi {
        let output_store = self.execution_stores.get_output_store(0);
        export_abi_by_component(&output_store, component_address).expect("Failed to export ABI")
    }

    pub fn update_resource_auth(
        &mut self,
        function: &str,
        auth: ResourceAddress,
        token: ResourceAddress,
        set_auth: ResourceAddress,
        account: ComponentAddress,
        signer_public_key: EcdsaPublicKey,
    ) {
        let package = self.extract_and_publish_package("resource_creator");
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .create_proof_from_account(auth, account)
            .call_function(package, "ResourceCreator", function, args!(token, set_auth))
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        self.execute_manifest(manifest, vec![signer_public_key])
            .expect_success();
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
    ) {
        let mint_auth = self.create_non_fungible_resource(account);
        let burn_auth = self.create_non_fungible_resource(account);
        let withdraw_auth = self.create_non_fungible_resource(account);
        let admin_auth = self.create_non_fungible_resource(account);

        let package = self.extract_and_publish_package("resource_creator");
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_token",
                args!(mint_auth, burn_auth, withdraw_auth, admin_auth),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        (
            receipt.new_resource_addresses[0],
            mint_auth,
            burn_auth,
            withdraw_auth,
            admin_auth,
        )
    }

    pub fn create_restricted_burn_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);
        let package = self.extract_and_publish_package("resource_creator");
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_burn",
                args!(auth_resource_address),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let package = self.extract_and_publish_package("resource_creator");
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_transfer",
                args![auth_resource_address],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        let package = self.extract_and_publish_package("resource_creator");
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_function(
                package,
                "ResourceCreator",
                "create_non_fungible_fixed",
                args!(),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_success();
        receipt.new_resource_addresses[0]
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let package = self.extract_and_publish_package("resource_creator");
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_function(
                package,
                "ResourceCreator",
                "create_fungible_fixed",
                args!(amount, divisibility),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.new_resource_addresses[0]
    }

    pub fn instantiate_component(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<String>,
        account: ComponentAddress,
        signer_public_key: EcdsaPublicKey,
    ) -> ComponentAddress {
        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_function_with_abi(
                package_address,
                blueprint_name,
                function_name,
                args,
                Some(account),
                &self.export_abi(package_address, blueprint_name),
            )
            .unwrap()
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![signer_public_key]);
        receipt.new_component_addresses[0]
    }

    pub fn set_current_epoch(&mut self, epoch: u64) {
        self.kernel_call(true, |kernel| {
            kernel
                .invoke_method(
                    Receiver::Ref(RENodeId::System),
                    FnIdentifier::Native(NativeFnIdentifier::System(SystemFnIdentifier::SetEpoch)),
                    ScryptoValue::from_typed(&SystemSetEpochInput { epoch }),
                )
                .unwrap()
        });
    }

    pub fn get_current_epoch(&mut self) -> u64 {
        let current_epoch: ScryptoValue = self.kernel_call(false, |kernel| {
            kernel
                .invoke_method(
                    Receiver::Ref(RENodeId::System),
                    FnIdentifier::Native(NativeFnIdentifier::System(
                        SystemFnIdentifier::GetCurrentEpoch,
                    )),
                    ScryptoValue::from_typed(&SystemGetCurrentEpochInput {}),
                )
                .unwrap()
        });
        scrypto_decode(&current_epoch.raw).unwrap()
    }

    /// Performs a kernel call through a kernel with `is_system = true`.
    fn kernel_call<F>(&mut self, is_system: bool, fun: F) -> ScryptoValue
    where
        F: FnOnce(
            &mut Kernel<DefaultWasmEngine, DefaultWasmInstance, SystemLoanFeeReserve>,
        ) -> ScryptoValue,
    {
        let tx_hash = hash(self.next_transaction_nonce.to_string());
        let mut substate_store = self.execution_stores.get_output_store(0);
        let mut track = Track::new(&substate_store, SystemLoanFeeReserve::default());
        let mut execution_trace = ExecutionTrace::new();

        let mut kernel = Kernel::new(
            tx_hash,
            Vec::new(),
            is_system,
            DEFAULT_MAX_CALL_DEPTH,
            &mut track,
            &mut self.wasm_engine,
            &mut self.wasm_instrumenter,
            WasmMeteringParams::new(InstructionCostRules::tiered(1, 5, 10, 5000), 512), // TODO: add to ExecutionConfig
            &mut execution_trace,
            Vec::new(),
        );

        // Invoke the system
        let output: ScryptoValue = fun(&mut kernel);

        // Commit
        self.next_transaction_nonce += 1;
        let receipt = track.finalize(Ok(Vec::new()));
        receipt.state_updates.commit(&mut substate_store);

        output
    }
}

pub fn is_auth_error(e: &RuntimeError) -> bool {
    matches!(
        e,
        RuntimeError::ModuleError(ModuleError::AuthorizationError {
            authorization: _,
            function: _,
            error: ::radix_engine::model::MethodAuthorizationError::NotAuthorized
        })
    )
}

#[macro_export]
macro_rules! assert_invoke_error {
    ($result:expr, $pattern:pat) => {{
        let matches = match &$result {
            radix_engine::transaction::TransactionStatus::Failed(
                radix_engine::engine::RuntimeError::KernelError(
                    radix_engine::engine::KernelError::WasmInvokeError(e),
                ),
            ) => {
                matches!(e.as_ref(), $pattern)
            }
            _ => false,
        };

        if !matches {
            panic!("Expected invoke error but got: {:?}", $result);
        }
    }};
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

pub fn abi_single_fn_any_input_void_output(
    blueprint_name: &str,
    function_name: &str,
) -> HashMap<String, BlueprintAbi> {
    let mut blueprint_abis = HashMap::new();
    blueprint_abis.insert(
        blueprint_name.to_string(),
        BlueprintAbi {
            structure: Type::Unit,
            fns: vec![Fn {
                ident: function_name.to_string(),
                mutability: Option::None,
                input: Type::Struct {
                    name: "Any".to_string(),
                    fields: Fields::Named { named: vec![] },
                },
                output: Type::Unit,
                export_name: format!("{}_{}", blueprint_name, function_name),
            }],
        },
    );
    blueprint_abis
}
