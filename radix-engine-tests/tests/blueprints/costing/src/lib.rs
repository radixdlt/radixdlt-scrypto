use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct TestNFData {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod costing {
    struct CostingTest {
        resource_manager: ResourceManager,
    }

    impl CostingTest {
        pub fn init() -> Global<CostingTest> {
            let resource_manager =
                ResourceBuilder::new_ruid_non_fungible::<TestNFData>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            Self { resource_manager }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn mint_1_nft(&mut self) -> Bucket {
            self.resource_manager.mint_ruid_non_fungible(TestNFData {
                name: "Test".to_owned(),
                available: true,
            })
        }

        pub fn mint_n_nfts(&mut self, n: u8) -> Bucket {
            let mut nfts = Bucket::new(self.resource_manager.address());
            for _ in 0..n {
                nfts.put(self.mint_1_nft());
            }
            nfts
        }
    }
}
