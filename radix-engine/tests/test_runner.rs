use radix_engine::ledger::*;
use radix_engine::model::{Component, Receipt, SignedTransaction};
use radix_engine::transaction::*;
use scrypto::abi;
use scrypto::prelude::*;

pub struct TestRunner<'l> {
    executor: TransactionExecutor<'l, InMemorySubstateStore>,
}

impl<'l> TestRunner<'l> {
    pub fn new(ledger: &'l mut InMemorySubstateStore) -> Self {
        let executor = TransactionExecutor::new(ledger, true);

        Self { executor }
    }

    pub fn new_transaction_builder(&self) -> TransactionBuilder {
        TransactionBuilder::new()
    }

    pub fn new_key_pair(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey) {
        self.executor.new_key_pair()
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
        self.executor.new_account_with_auth_rule(withdraw_auth)
    }

    pub fn new_account(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) {
        self.executor.new_account()
    }

    pub fn validate_and_execute(&mut self, transaction: &SignedTransaction) -> Receipt {
        self.executor.validate_and_execute(transaction).unwrap()
    }

    pub fn publish_package(&mut self, name: &str) -> PackageAddress {
        self.executor.publish_package(&Self::compile(name)).unwrap()
    }

    pub fn compile(name: &str) -> Vec<u8> {
        compile_package!(format!("./tests/{}", name))
    }

    pub fn component(&self, component_address: ComponentAddress) -> Component {
        self.executor
            .substate_store()
            .get_decoded_substate(&component_address)
            .map(|(component, _)| component)
            .unwrap()
    }

    pub fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> abi::Blueprint {
        self.executor
            .export_abi(package_address, blueprint_name)
            .unwrap()
    }

    pub fn export_abi_by_component(&self, component_address: ComponentAddress) -> abi::Blueprint {
        self.executor
            .export_abi_by_component(component_address)
            .unwrap()
    }

    pub fn get_nonce<PKS: AsRef<[EcdsaPublicKey]>>(&self, intended_signers: PKS) -> u64 {
        self.executor.get_nonce(intended_signers)
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
        let transaction = TransactionBuilder::new()
            .create_proof_from_account(auth, account.2)
            .call_function(
                package,
                "ResourceCreator",
                function,
                vec![scrypto_encode(&token), scrypto_encode(&set_auth)],
            )
            .call_method_with_all_resources(account.2, "deposit_batch")
            .build(self.executor.get_nonce([account.0.clone()]))
            .sign([account.1]);
        let result = self
            .executor
            .validate_and_execute(&transaction)
            .unwrap()
            .result;
        result.expect("Should be okay");
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
        let transaction = TransactionBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_token",
                vec![
                    scrypto_encode(&mint_auth),
                    scrypto_encode(&burn_auth),
                    scrypto_encode(&withdraw_auth),
                    scrypto_encode(&admin_auth),
                ],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(self.executor.get_nonce([]))
            .sign([]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
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
        let transaction = TransactionBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_burn",
                vec![scrypto_encode(&auth_resource_address)],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(self.executor.get_nonce([]))
            .sign([]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_transfer",
                vec![scrypto_encode(&auth_resource_address)],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(self.executor.get_nonce([]))
            .sign([]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_non_fungible_fixed",
                vec![],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(self.executor.get_nonce([]))
            .sign([]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
        receipt.new_resource_addresses[0]
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_fungible_fixed",
                args![amount, divisibility],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(self.executor.get_nonce([]))
            .sign([]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
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
        let transaction = self
            .new_transaction_builder()
            .call_function_with_abi(
                package_address,
                blueprint_name,
                function_name,
                args,
                Some(account),
                &self
                    .executor
                    .export_abi(package_address, blueprint_name)
                    .unwrap(),
            )
            .unwrap()
            .call_method_with_all_resources(account, "deposit_batch")
            .build(self.executor.get_nonce([pk]))
            .sign([sk]);
        let receipt = self.validate_and_execute(&transaction);
        receipt.new_component_addresses[0]
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
