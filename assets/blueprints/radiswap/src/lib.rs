use scrypto::prelude::*;

#[blueprint]
mod radiswap {
    struct Radiswap {
        pool_component: Global<TwoResourcePool>,
    }

    impl Radiswap {
        pub fn new(
            owner_role: OwnerRole,
            resource_address1: ResourceAddress,
            resource_address2: ResourceAddress,
        ) -> Global<Radiswap> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Radiswap::blueprint_id());
            let global_component_caller_badge =
                NonFungibleGlobalId::global_caller_badge(component_address);

            // Creating a new pool will check the following for us:
            // 1. That both resources are not the same.
            // 2. That none of the resources are non-fungible
            let pool_component = Blueprint::<TwoResourcePool>::instantiate(
                owner_role.clone(),
                rule!(require(global_component_caller_badge)),
                (resource_address1, resource_address2),
                None,
            );

            Self { pool_component }
                .instantiate()
                .prepare_to_globalize(owner_role)
                .with_address(address_reservation)
                .globalize()
        }

        pub fn add_liquidity(
            &mut self,
            resource1: Bucket,
            resource2: Bucket,
        ) -> (Bucket, Option<Bucket>) {
            // All the checks for correctness of buckets and everything else is handled by the pool
            // component! Just pass it the resources and it will either return the pool units back
            // if it succeeds or abort on failure.
            self.pool_component.contribute((resource1, resource2))
        }

        /// This method does not need to be here - the pool units are redeemable without it by the
        /// holders of the pool units directly from the pool. In this case this is just a nice proxy
        /// so that users are only interacting with one component and do not need to know about the
        /// address of Radiswap and the address of the Radiswap pool.
        pub fn remove_liquidity(&mut self, pool_units: Bucket) -> (Bucket, Bucket) {
            self.pool_component.redeem(pool_units)
        }

        pub fn swap(&mut self, input_bucket: Bucket) -> Bucket {
            let mut reserves = self.vault_reserves();

            let input_amount = input_bucket.amount();

            let input_reserves = reserves
                .remove(&input_bucket.resource_address())
                .expect("Resource does not belong to the pool");
            let (output_resource_address, output_reserves) = reserves.into_iter().next().unwrap();

            let output_amount = input_amount
                .safe_mul(output_reserves)
                .unwrap()
                .safe_div(input_reserves.safe_add(input_amount).unwrap())
                .unwrap();

            // NOTE: It's the responsibility of the user of the pool to do the appropriate rounding
            // before calling the withdraw method.

            self.deposit(input_bucket);
            self.withdraw(output_resource_address, output_amount)
        }

        fn vault_reserves(&self) -> BTreeMap<ResourceAddress, Decimal> {
            self.pool_component.get_vault_amounts()
        }

        fn deposit(&mut self, bucket: Bucket) {
            self.pool_component.protected_deposit(bucket)
        }

        fn withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket {
            self.pool_component.protected_withdraw(
                resource_address,
                amount,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            )
        }
    }
}
