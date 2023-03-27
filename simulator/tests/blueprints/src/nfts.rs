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
            ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Cars!")
                .metadata("description", "Fast Cars")
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
        }
    }
}
