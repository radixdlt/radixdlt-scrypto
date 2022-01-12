use scrypto::prelude::*;

#[derive(NftData)]
pub struct PresentList {
    #[scrypto(mutable)]
    presents: Vec<String>
}

blueprint! {
    struct PresentListWithNFT {
        list_minter: Vault,
        list_def: ResourceDef,
        nb_lists: u128
    }

    impl PresentListWithNFT {
        pub fn new() -> Component {
            // Create a badge that will allow the component to
            // mint new list NFTs
            let list_minter: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                        .initial_supply_fungible(1);

            // Create the definition of the NFT
            let list_resource_def: ResourceDef = ResourceBuilder::new_non_fungible()
                                                    .metadata("name", "Christmas List")
                                                    .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                                                    .badge(list_minter.resource_def(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA)
                                                    .no_initial_supply();

            // Store all required information info the component's state
            Self {
                list_minter: Vault::with_bucket(list_minter),
                list_def: list_resource_def,
                nb_lists: 0
            }
            .instantiate()
        }
        
        // Allow the user to start a new christmas list.
        // It generates a list NFT that will contain the list items
        pub fn start_new_list(&mut self) -> Bucket {
            // Mint a new christmas list badge and return it to the caller
            self.list_minter.authorize(|badge| {
                self.list_def.mint_nft(self.nb_lists, PresentList { presents: Vec::new() }, badge)
            })
        }

        // Add a new present to the list
        pub fn add(&self, present_name: String, list: BucketRef) {
            let list_id = list.get_nft_id();
            let mut list_data: PresentList = self.list_def.get_nft_data(list.get_nft_id());
            list.drop();

            // Make sure that the present is not already inside the user's list
            assert!(!list_data.presents.contains(&present_name), "Present already on the list !");

            list_data.presents.push(present_name);

            // Update the list with the newly added present
            self.list_minter.authorize(|badge| {
                self.list_def.update_nft_data(list_id, list_data, badge)
            });

            info!("Present added to your list !");
        }

        // Remove a present in the list
        pub fn remove(&self, present_name: String, list: BucketRef) {
            let list_id = list.get_nft_id();
            let mut list_data: PresentList = self.list_def.get_nft_data(list.get_nft_id());
            list.drop();

            // Make sure that the present is not already inside the user's list
            assert!(list_data.presents.contains(&present_name), "Present not on the list !");
        
            // Find the index of the present to remove
            let index = list_data.presents.iter().position(|x| *x == present_name).unwrap();
            list_data.presents.remove(index);

            // Update the list with the present removed
            self.list_minter.authorize(|badge| {
                self.list_def.update_nft_data(list_id, list_data, badge);
            })
        }

        // Display the presents stored in the NFT
        pub fn display_list(&self, list: BucketRef) {
            let list_data: PresentList = self.list_def.get_nft_data(list.get_nft_id());
            list.drop();

            info!("==== Christmas list content");
            for item in list_data.presents.iter() {
                info!("{}", item);
            }
        }
    }
}