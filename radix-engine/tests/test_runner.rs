use radix_engine::engine::{Receipt, TransactionExecutor};
use radix_engine::ledger::*;
use radix_engine::model::{export_abi, export_abi_by_component, extract_package, Component};
use radix_engine::wasm::DefaultWasmEngine;
use scrypto::prelude::*;
use scrypto::{abi, call_data};
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::TestTransaction;

pub struct TestRunner {
    substate_store: InMemorySubstateStore,
    wasm_engine: DefaultWasmEngine,
    nonce: u64,
    trace: bool,
}

impl TestRunner {
    pub fn new(trace: bool) -> Self {
        Self {
            substate_store: InMemorySubstateStore::with_bootstrap(),
            wasm_engine: DefaultWasmEngine::new(),
            nonce: 1,
            trace,
        }
    }

    pub fn new_key_pair(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey) {
        let private_key = EcdsaPrivateKey::from_u64(self.next_nonce()).unwrap();
        let public_key = private_key.public_key();
        (public_key, private_key)
    }

    pub fn new_key_pair_with_pk_address(
        &mut self,
    ) -> (EcdsaPublicKey, EcdsaPrivateKey, NonFungibleAddress) {
        let (pk, sk) = self.new_key_pair();
        (
            pk,
            sk,
            NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(pk.to_vec())),
        )
    }

    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &AccessRule) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .call_method(SYSTEM_COMPONENT, call_data!(free_xrd()))
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.new_account_with_resource(withdraw_auth, bucket_id)
            })
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.new_component_addresses[0]
    }

    pub fn new_account(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) {
        let (public_key, private_key, pk_address) = self.new_key_pair_with_pk_address();
        let withdraw_auth = rule!(require(pk_address));

        (
            public_key,
            private_key,
            self.new_account_with_auth_rule(&withdraw_auth),
        )
    }

    pub fn publish_package(&mut self, name: &str) -> PackageAddress {
        self.publish_package_with_code(compile_package!(format!("./tests/{}", name)))
    }

    pub fn publish_package_with_code(&mut self, code: Vec<u8>) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .publish_package(extract_package(code).expect("Failed to extract package"))
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.new_package_addresses[0]
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifest,
        signers: Vec<EcdsaPublicKey>,
    ) -> Receipt {
        let transaction = TestTransaction::new(manifest, self.next_nonce(), signers);

        TransactionExecutor::new(&mut self.substate_store, &mut self.wasm_engine, self.trace)
            .execute(&transaction)
    }

    pub fn inspect_component(&self, component_address: ComponentAddress) -> Component {
        self.substate_store
            .get_decoded_substate(&component_address)
            .map(|(component, _)| component)
            .unwrap()
    }

    pub fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> abi::Blueprint {
        export_abi(&self.substate_store, package_address, blueprint_name)
            .expect("Failed to export ABI")
    }

    pub fn export_abi_by_component(&self, component_address: ComponentAddress) -> abi::Blueprint {
        export_abi_by_component(&self.substate_store, component_address)
            .expect("Failed to export ABI")
    }

    pub fn set_auth(
        &mut self,
        account: (&EcdsaPublicKey, &EcdsaPrivateKey, ComponentAddress),
        function: &str,
        auth: ResourceAddress,
        token: ResourceAddress,
        set_auth: ResourceAddress,
    ) {
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .create_proof_from_account(auth, account.2)
            .call_function(
                package,
                "ResourceCreator",
                call_data!(function.to_string(), token, set_auth),
            )
            .call_method_with_all_resources(account.2, "deposit_batch")
            .build();
        let signers = vec![account.0.clone()];
        self.execute_manifest(manifest, signers)
            .result
            .expect("Should be okay");
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

        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                call_data!(create_restricted_token(
                    mint_auth,
                    burn_auth,
                    withdraw_auth,
                    admin_auth
                )),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let signers = vec![];
        let receipt = self.execute_manifest(manifest, signers);
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
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                call_data!(create_restricted_burn(auth_resource_address)),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let signers = vec![];
        let receipt = self.execute_manifest(manifest, signers);
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                call_data![create_restricted_transfer(auth_resource_address)],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let signers = vec![];
        let receipt = self.execute_manifest(manifest, signers);
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                call_data!(create_non_fungible_fixed()),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let signers = vec![];
        let receipt = self.execute_manifest(manifest, signers);
        receipt.result.expect("Should be okay.");
        receipt.new_resource_addresses[0]
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                call_data!(create_fungible_fixed(amount, divisibility)),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let signers = vec![];
        let receipt = self.execute_manifest(manifest, signers);
        receipt.new_resource_addresses[0]
    }

    pub fn instantiate_component(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<String>,
        account: ComponentAddress,
        pk: EcdsaPublicKey,
        sk: &EcdsaPrivateKey,
    ) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
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
        let signers = vec![pk];
        let receipt = self.execute_manifest(manifest, signers);
        receipt.new_component_addresses[0]
    }

    fn next_nonce(&mut self) -> u64 {
        let nonce = self.nonce;
        self.nonce += 1;
        nonce
    }
}

#[macro_export]
macro_rules! assert_auth_error {
    ($error:expr) => {{
        if !matches!(
            $error,
            RuntimeError::AuthorizationError {
                authorization: _,
                function: _,
                error: ::radix_engine::model::MethodAuthorizationError::NotAuthorized
            }
        ) {
            panic!("Expected auth error but got: {:?}", $error);
        }
    }};
}

#[macro_export]
macro_rules! assert_invoke_error {
    ($result:expr, $pattern:pat) => {{
        let matches = match &$result {
            Err(radix_engine::engine::RuntimeError::InvokeError(e)) => {
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
        wat.replace("${memcpy}", include_str!("wasm/snippets/memcpy.wat"))
            .replace("${memmove}", include_str!("wasm/snippets/memmove.wat"))
            .replace("${memset}", include_str!("wasm/snippets/memset.wat"))
            .replace("${buffer}", include_str!("wasm/snippets/buffer.wat")),
    )
    .expect("Failed to compiled WAT into WASM")
}
