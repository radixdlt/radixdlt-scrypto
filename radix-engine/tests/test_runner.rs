use radix_engine::ledger::*;
use radix_engine::model::Receipt;
use radix_engine::model::Transaction;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub struct TestRunner<'l> {
    executor: TransactionExecutor<'l, InMemorySubstateStore>,
}

impl<'l> TestRunner<'l> {
    pub fn new(ledger: &'l mut InMemorySubstateStore) -> Self {
        let executor = TransactionExecutor::new(ledger, true);

        Self { executor }
    }

    pub fn new_transaction_builder(
        &self,
    ) -> TransactionBuilder<TransactionExecutor<InMemorySubstateStore>> {
        TransactionBuilder::new(&self.executor)
    }

    pub fn new_public_key_and_non_fungible_address(
        &mut self,
    ) -> (EcdsaPublicKey, NonFungibleAddress) {
        let key = self.executor.new_public_key();
        let id = NonFungibleId::new(key.to_vec());
        let non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, id);
        (key, non_fungible_address)
    }

    pub fn new_account(&mut self, withdraw_auth: &ProofRule) -> ComponentId {
        self.executor.new_account(withdraw_auth)
    }

    pub fn new_public_key_with_account(&mut self) -> (EcdsaPublicKey, ComponentId) {
        self.executor.new_public_key_with_account()
    }

    pub fn run(&mut self, transaction: Transaction) -> Receipt {
        self.executor.run(transaction).unwrap()
    }

    pub fn publish_package(&mut self, name: &str) -> PackageId {
        self.executor.publish_package(&Self::compile(name)).unwrap()
    }

    pub fn compile(name: &str) -> Vec<u8> {
        compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
    }

    pub fn create_restricted_mint_token(
        &mut self,
        account: ComponentId,
    ) -> (ResourceDefId, ResourceDefId) {
        let auth_resource_def_id = self.create_non_fungible_resource(account);

        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new(&self.executor)
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_mint",
                vec![scrypto_encode(&auth_resource_def_id)],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(vec![])
            .unwrap();
        let receipt = self.executor.run(transaction).unwrap();
        (auth_resource_def_id, receipt.new_resource_def_ids[0])
    }

    pub fn create_restricted_burn_token(
        &mut self,
        account: ComponentId,
    ) -> (ResourceDefId, ResourceDefId) {
        let auth_resource_def_id = self.create_non_fungible_resource(account);
        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new(&self.executor)
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_burn",
                vec![scrypto_encode(&auth_resource_def_id)],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(vec![])
            .unwrap();
        let receipt = self.executor.run(transaction).unwrap();
        (auth_resource_def_id, receipt.new_resource_def_ids[0])
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentId,
    ) -> (ResourceDefId, ResourceDefId) {
        let auth_resource_def_id = self.create_non_fungible_resource(account);

        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new(&self.executor)
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_transfer",
                vec![scrypto_encode(&auth_resource_def_id)],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(vec![])
            .unwrap();
        let receipt = self.executor.run(transaction).unwrap();
        (auth_resource_def_id, receipt.new_resource_def_ids[0])
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentId) -> ResourceDefId {
        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new(&self.executor)
            .call_function(
                package,
                "ResourceCreator",
                "create_non_fungible_fixed",
                vec![],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(vec![])
            .unwrap();
        let receipt = self.executor.run(transaction).unwrap();
        receipt.new_resource_def_ids[0]
    }

    pub fn create_fungible_resource(&mut self, account: ComponentId) -> ResourceDefId {
        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new(&self.executor)
            .call_function(package, "ResourceCreator", "create_fungible_fixed", vec![])
            .call_method_with_all_resources(account, "deposit_batch")
            .build(vec![])
            .unwrap();
        let receipt = self.executor.run(transaction).unwrap();
        receipt.new_resource_def_ids[0]
    }

    pub fn instantiate_component(
        &mut self,
        package_id: PackageId,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<String>,
        account: ComponentId,
        key: EcdsaPublicKey,
    ) -> ComponentId {
        let transaction = self
            .new_transaction_builder()
            .parse_args_and_call_function(
                package_id,
                blueprint_name,
                function_name,
                args,
                Some(account),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(vec![key])
            .unwrap();
        let receipt = self.run(transaction);
        receipt.new_component_ids[0]
    }
}
