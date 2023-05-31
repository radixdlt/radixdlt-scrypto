use scrypto::blueprints::pool::*;
use scrypto::prelude::*;

// =================================================================================================
// TODO: The interface of this can be made better once we have a way to generate stubs for native
//       blueprints such that we're storing `Global<Radiswap>` instead of `Global<AnyComponent>`
//       and this also applies to function and method calls on the package and component.
// =================================================================================================

#[blueprint]
mod radiswap {
    struct Radiswap {
        // TODO: We need a stub for native blueprints so that we're not using `AnyComponent`.
        /// The liquidity pool used by Radiswap and that manages all of the pool unit tokens.
        pool_component: Global<AnyComponent>,
    }

    impl Radiswap {
        pub fn new(
            resource_address1: ResourceAddress,
            resource_address2: ResourceAddress,
        ) -> Global<Radiswap> {
            let component_address = Runtime::preallocate_global_component_address();
            let global_component_caller_badge =
                NonFungibleGlobalId::global_caller_badge(component_address);

            // Creating a new pool will check the following for us:
            // 1. That both resources are not the same.
            // 2. That none of the resources are non-fungible
            let pool_component = Runtime::call_function::<_, _, Global<AnyComponent>>(
                POOL_PACKAGE,
                "TwoResourcePool",
                TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                scrypto_encode(&TwoResourcePoolInstantiateInput {
                    pool_manager_rule: rule!(require(global_component_caller_badge)),
                    resource_addresses: (resource_address1, resource_address2),
                })
                .unwrap(),
            );

            Self { pool_component }
                .instantiate()
                .globalize_at_address(component_address)
        }

        pub fn add_liquidity(
            &mut self,
            resource1: Bucket,
            resource2: Bucket,
        ) -> (Bucket, Option<Bucket>) {
            // All the checks for correctness of buckets and everything else is handled by the pool
            // component! Just pass it the resources and it will either return the pool units back
            // if it succeeds or abort on failure.
            self.pool_component
                .call::<TwoResourcePoolContributeInput, TwoResourcePoolContributeOutput>(
                    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    &TwoResourcePoolContributeInput {
                        buckets: (resource1, resource2),
                    },
                )
        }

        /// This method does not need to be here - the pool units are redeemable without it by the
        /// holders of the pool units directly from the pool. In this case this is just a nice proxy
        /// so that users are only interacting with one component and do not need to know about the
        /// address of Radiswap and the address of the Radiswap pool.
        pub fn remove_liquidity(&mut self, pool_units: Bucket) -> (Bucket, Bucket) {
            self.pool_component
                .call::<TwoResourcePoolRedeemInput, TwoResourcePoolRedeemOutput>(
                    TWO_RESOURCE_POOL_REDEEM_IDENT,
                    &TwoResourcePoolRedeemInput { bucket: pool_units },
                )
        }

        pub fn swap(&mut self, input_bucket: Bucket) -> Bucket {
            let mut reserves = self.vault_reserves();

            let input_amount = input_bucket.amount();

            let input_reserves = reserves
                .remove(&input_bucket.resource_address())
                .expect("Resource does not belong to the pool");
            let (output_resource_address, output_reserves) = reserves.into_iter().next().unwrap();

            let output_amount = (input_amount * output_reserves) / (input_reserves + input_amount);

            self.deposit(input_bucket);
            self.withdraw(output_resource_address, output_amount)
        }

        fn vault_reserves(&self) -> BTreeMap<ResourceAddress, Decimal> {
            self.pool_component
                .call::<TwoResourcePoolGetVaultAmountsInput, TwoResourcePoolGetVaultAmountsOutput>(
                    TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
                    &TwoResourcePoolGetVaultAmountsInput,
                )
        }

        fn deposit(&mut self, bucket: Bucket) {
            self.pool_component
                .call::<TwoResourcePoolProtectedDepositInput, TwoResourcePoolProtectedDepositOutput>(
                    TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    &TwoResourcePoolProtectedDepositInput { bucket },
                )
        }

        fn withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket {
            self.pool_component
                .call::<TwoResourcePoolProtectedWithdrawInput, TwoResourcePoolProtectedWithdrawOutput>(
                    TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                    &TwoResourcePoolProtectedWithdrawInput {
                        resource_address,
                        amount,
                    }
                )
        }
    }
}
