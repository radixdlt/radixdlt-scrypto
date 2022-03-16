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
                vec![auth_resource_def_id.to_string()],
                Some(account),
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
                Some(account),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build(vec![])
            .unwrap();
        let receipt = self.executor.run(transaction).unwrap();
        receipt.new_resource_def_ids[0]
    }
}
