use scrypto::prelude::*;

#[derive(NonFungibleData)]
struct Car {
    name: String,
    manufacturer: String,
}

blueprint! {
    struct Foo {}

    impl Foo {
        pub fn nfts() -> Bucket {
            ResourceBuilder::new_non_fungible::<u128>()
                .metadata("name", "Cars!")
                .metadata("description", "Fast Cars")
                .initial_supply(vec![
                    (
                        0u128,
                        Car {
                            manufacturer: "Ford".to_string(),
                            name: "Raptor".to_string(),
                        },
                    ),
                    (
                        1u128,
                        Car {
                            manufacturer: "Toyota".to_string(),
                            name: "Camry".to_string(),
                        },
                    ),
                    (
                        2u128,
                        Car {
                            manufacturer: "Nissan".to_string(),
                            name: "Altima".to_string(),
                        },
                    ),
                ])
        }
    }
}
