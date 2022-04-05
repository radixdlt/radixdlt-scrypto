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
            NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(pk.to_vec())),
        )
    }

    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &ProofRule) -> ComponentId {
        self.executor.new_account_with_auth_rule(withdraw_auth)
    }

    pub fn new_account(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey, ComponentId) {
        self.executor.new_account()
    }

    pub fn validate_and_execute(&mut self, transaction: &Transaction) -> Receipt {
        self.executor.validate_and_execute(transaction).unwrap()
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
            .build(&[])
            .unwrap()
            .sign(&[]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
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
            .build(&[])
            .unwrap()
            .sign(&[]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
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
            .build(&[])
            .unwrap()
            .sign(&[]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
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
            .build(&[])
            .unwrap()
            .sign(&[]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
        receipt.new_resource_def_ids[0]
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentId,
    ) -> ResourceDefId {
        let package = self.publish_package("resource_creator");
        let transaction = TransactionBuilder::new(&self.executor)
            .call_function(
                package,
                "ResourceCreator",
                "create_fungible_fixed",
                args![amount, divisibility],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(&[])
            .unwrap()
            .sign(&[]);
        let receipt = self.executor.validate_and_execute(&transaction).unwrap();
        receipt.new_resource_def_ids[0]
    }

    pub fn instantiate_component(
        &mut self,
        package_id: PackageId,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<String>,
        account: ComponentId,
        pk: EcdsaPublicKey,
        sk: EcdsaPrivateKey,
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
            .build(&[pk])
            .unwrap()
            .sign(&[sk]);
        let receipt = self.validate_and_execute(&transaction);
        receipt.new_component_ids[0]
    }
}
