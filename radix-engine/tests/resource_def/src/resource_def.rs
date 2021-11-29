use scrypto::prelude::*;

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_fungible() -> (Bucket, ResourceDef) {
            let badge = ResourceBuilder::new_fungible()
                .granularity(19)
                .flags(FREELY_TRANSFERABLE)
                .initial_supply(NewSupply::fungible(1));
            let token_resource_def = ResourceBuilder::new_fungible()
                .metadata("name", "TestToken")
                .flags(FREELY_TRANSFERABLE | MINTABLE | BURNABLE)
                .badge(badge.resource_address(), MAY_MINT | MAY_BURN)
                .no_initial_supply();
            (badge, token_resource_def)
        }

        pub fn create_fungible_should_fail() -> (Bucket, Bucket) {
            let bucket = ResourceBuilder::new_fungible()
                .granularity(19)
                .flags(FREELY_TRANSFERABLE)
                .initial_supply(NewSupply::fungible(1));
            (bucket.take(Decimal::from_str("0.1").unwrap()), bucket)
        }

        pub fn query() -> (Bucket, HashMap<String, String>, u8, u16, u16, Decimal) {
            let (badge, resource_def) = Self::create_fungible();
            (
                badge,
                resource_def.metadata(),
                resource_def.granularity(),
                resource_def.flags(),
                resource_def.mutable_flags(),
                resource_def.total_supply(),
            )
        }

        pub fn burn() -> Bucket {
            let (badge, resource_def) = Self::create_fungible();
            let bucket = resource_def.mint(1, badge.present());
            resource_def.burn(bucket, badge.present());
            badge
        }
    }
}
