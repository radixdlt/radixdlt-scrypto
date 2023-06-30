use scrypto::prelude::*;

#[blueprint]
mod execution_trace_test {
    struct ExecutionTraceTest {
        vault: Vault,
    }

    impl ExecutionTraceTest {
        pub fn transfer_resource_between_two_components(
            amount: u8,
        ) -> (
            ResourceAddress,
            Global<ExecutionTraceTest>,
            Global<ExecutionTraceTest>,
        ) {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .mint_initial_supply(1000000);

            let resource_address = bucket.resource_address();

            let source_component = ExecutionTraceTest {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            let target_component = ExecutionTraceTest {
                vault: Vault::new(resource_address),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            let transfer_bucket: Bucket = source_component.take(amount);
            target_component.put(transfer_bucket);

            (resource_address, source_component, target_component)
        }

        pub fn take(&mut self, amount: u8) -> Bucket {
            self.vault.take(amount)
        }

        pub fn put(&mut self, b: Bucket) {
            self.vault.put(b)
        }

        pub fn create_and_fund_a_component(xrd: Vec<Bucket>) -> Global<ExecutionTraceTest> {
            let vault = Vault::with_bucket(xrd.into_iter().nth(0).unwrap());
            ExecutionTraceTest { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn test_lock_contingent_fee(&mut self) {
            self.vault.as_fungible().lock_contingent_fee(dec!("500"));
        }
    }
}
