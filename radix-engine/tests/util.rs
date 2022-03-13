use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub struct TestUtil {
}

impl TestUtil {
    pub fn compile(name: &str) -> Vec<u8> {
        compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
    }

    pub fn create_restricted_transfer_token(
        executor: &mut TransactionExecutor<InMemorySubstateStore>,
        account: ComponentId,
    ) -> (ResourceDefId, ResourceDefId) {
        let auth_resource_def_id = Self::create_non_fungible_resource(executor, account);

        let package = executor
            .publish_package(&Self::compile("resource_creator"))
            .unwrap();
        let transaction = TransactionBuilder::new(executor)
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
        let receipt = executor.run(transaction).unwrap();
        (auth_resource_def_id, receipt.new_resource_def_ids[0])
    }

    pub fn create_non_fungible_resource(
        executor: &mut TransactionExecutor<InMemorySubstateStore>,
        account: ComponentId,
    ) -> ResourceDefId {
        let package = executor
            .publish_package(&Self::compile("resource_creator"))
            .unwrap();
        let transaction = TransactionBuilder::new(executor)
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
        let receipt = executor.run(transaction).unwrap();
        receipt.new_resource_def_ids[0]
    }
}