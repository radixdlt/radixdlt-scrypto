use scrypto::prelude::*;

#[derive(NonFungibleData)]
struct Car {
    name: String,
    manufacturer: String,
}

#[blueprint]
mod blueprint {
    struct Foo {}

    impl Foo {
        pub fn nfts() -> Bucket {
            ResourceBuilder::new_non_fungible::<u128>()
                .metadata("name", "Cars!")
                .metadata("description", "Fast Cars")
                .initial_supply_uuid(vec![
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
