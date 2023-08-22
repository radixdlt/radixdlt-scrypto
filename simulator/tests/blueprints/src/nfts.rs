use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct Car {
    name: String,
    manufacturer: String,
}

#[blueprint]
mod foo {
    struct Foo {}

    impl Foo {
        pub fn nfts() -> Bucket {
            ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
                .metadata(metadata! {
                    init {
                        "name" => "Cars!".to_owned(), locked;
                        "description" => "Fast Cars".to_owned(), locked;
                    }
                })
                .mint_initial_supply(vec![
                    Car {
                        manufacturer: "Ford".to_string(),
                        name: "Raptor".to_string(),
                    },
                    Car {
                        manufacturer: "Toyota".to_string(),
                        name: "Camry".to_string(),
                    },
                    Car {
                        manufacturer: "Nissan".to_string(),
                        name: "Altima".to_string(),
                    },
                ])
                .into()
        }
    }
}
