use scrypto::prelude::*;

use crate::nf_data_with_global::nf_data_with_global::NonFungibleWithGlobalTest;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct NFDataWithGlobal {
    pub global: Global<NonFungibleWithGlobalTest>,
}

#[blueprint]
mod nf_data_with_global {
    struct NonFungibleWithGlobalTest {}

    impl NonFungibleWithGlobalTest {
        pub fn create_non_fungible_with_global() -> (Bucket, Bucket, Bucket) {
            let global = NonFungibleWithGlobalTest {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();

            // Create a mint badge
            let mint_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            // Create  resource with initial supply
            let bucket1 = ResourceBuilder::new_integer_non_fungible::<NFDataWithGlobal>(
                OwnerRole::None,
            )
            .metadata(metadata! {
                init {
                    "name" => "NFDataWithGlobal".to_owned(), locked;
                }
            })
            .mintable(mintable! {
                minter => rule!(require(mint_badge.resource_address())), locked;
                minter_updater => rule!(deny_all), locked;
            })
            .burnable(burnable! {
                burner => rule!(allow_all), locked;
                burner_updater => rule!(deny_all), locked;
            })
            .updatable_non_fungible_data(updatable_non_fungible_data! {
                non_fungible_data_updater => rule!(require(mint_badge.resource_address())), locked;
                non_fungible_data_updater_updater => rule!(deny_all), locked;
            })
            .mint_initial_supply([(1u64.into(), NFDataWithGlobal { global })]);

            // Mint a non-fungible
            let bucket2 = mint_badge.authorize(|| {
                bucket1
                    .resource_manager()
                    .mint_non_fungible(&NonFungibleLocalId::integer(2), NFDataWithGlobal { global })
            });

            (mint_badge, bucket1, bucket2)
        }
    }
}
