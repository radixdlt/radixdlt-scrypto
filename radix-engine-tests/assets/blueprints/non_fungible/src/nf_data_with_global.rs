use scrypto::prelude::*;

use self::nf_data_with_global::NonFungibleWithGlobalTest;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct NFDataWithGlobal {
    pub global: Global<NonFungibleWithGlobalTest>,
}

#[blueprint]
mod nf_data_with_global {
    struct NonFungibleWithGlobalTest {}

    impl NonFungibleWithGlobalTest {
        pub fn create_non_fungible_with_global() -> (Bucket, NonFungibleBucket, NonFungibleBucket) {
            let global = NonFungibleWithGlobalTest {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();

            // Create a mint badge
            let mint_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

            // Create resource with initial supply
            let bucket1 =
                ResourceBuilder::new_integer_non_fungible::<NFDataWithGlobal>(OwnerRole::None)
                    .metadata(metadata! {
                        init {
                            "name" => "NFDataWithGlobal".to_owned(), locked;
                        }
                    })
                    .mint_roles(mint_roles! {
                        minter => rule!(require(mint_badge.resource_address()));
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    })
                    .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                        non_fungible_data_updater => rule!(require(mint_badge.resource_address()));
                        non_fungible_data_updater_updater => rule!(deny_all);
                    })
                    .mint_initial_supply([(1u64.into(), NFDataWithGlobal { global })]);

            // Mint a non-fungible
            let bucket2 = mint_badge.as_fungible().authorize_with_amount(dec!(1), || {
                bucket1
                    .resource_manager()
                    .mint_non_fungible(&NonFungibleLocalId::integer(2), NFDataWithGlobal { global })
            });

            (mint_badge, bucket1, bucket2)
        }
    }
}
