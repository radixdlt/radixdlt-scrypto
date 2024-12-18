use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct DummyData {}

#[blueprint]
mod execution_trace_test {
    struct ExecutionTraceBp {
        vault: Vault,
    }

    impl ExecutionTraceBp {
        pub fn transfer_fungible_resource_between_two_components(
            amount: u8,
            use_take_advanced: bool,
        ) -> (
            ResourceAddress,
            Global<ExecutionTraceBp>,
            Global<ExecutionTraceBp>,
        ) {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .mint_initial_supply(1000000);

            let resource_address = bucket.resource_address();

            let source_component = ExecutionTraceBp {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            let target_component = ExecutionTraceBp {
                vault: Vault::new(resource_address),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            let transfer_bucket: Bucket = if use_take_advanced {
                source_component.take_advanced(amount)
            } else {
                source_component.take(amount)
            };
            target_component.put(transfer_bucket);

            (resource_address, source_component, target_component)
        }

        pub fn transfer_non_fungible_resource_between_two_components(
            amount: u8,
        ) -> (
            ResourceAddress,
            Global<ExecutionTraceBp>,
            Global<ExecutionTraceBp>,
        ) {
            let mut entries = vec![];
            for i in 1..amount as u64 + 1 {
                entries.push((i.into(), DummyData {}));
            }
            let bucket = ResourceBuilder::new_integer_non_fungible::<DummyData>(OwnerRole::None)
                .mint_initial_supply(entries);
            let resource_address = bucket.resource_address();

            let source_component = ExecutionTraceBp {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            let target_component = ExecutionTraceBp {
                vault: Vault::new(resource_address),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            let transfer_bucket: Bucket = source_component.take_non_fungibles(amount);
            target_component.put(transfer_bucket);

            (resource_address, source_component, target_component)
        }

        pub fn take(&mut self, amount: u8) -> Bucket {
            self.vault.take(amount)
        }

        pub fn take_advanced(&mut self, amount: u8) -> Bucket {
            self.vault.take_advanced(amount, WithdrawStrategy::Exact)
        }

        pub fn take_non_fungibles(&mut self, amount: u8) -> Bucket {
            let amount = amount as u64;
            let mut ids = indexset![];
            for i in 1..amount + 1 {
                ids.insert(NonFungibleLocalId::integer(i));
            }

            self.vault.as_non_fungible().take_non_fungibles(&ids).into()
        }

        pub fn put(&mut self, b: Bucket) {
            self.vault.put(b)
        }

        pub fn create_and_fund_a_component(xrd: Vec<Bucket>) -> Global<ExecutionTraceBp> {
            let vault = Vault::with_bucket(xrd.into_iter().nth(0).unwrap());
            ExecutionTraceBp { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn test_lock_contingent_fee(&mut self) {
            self.vault.as_fungible().lock_contingent_fee(dec!("500"));
        }
    }
}

