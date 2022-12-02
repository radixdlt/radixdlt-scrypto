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
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Cars!")
                .metadata("description", "Fast Cars")
                .initial_supply(vec![
                    (
                        NonFungibleId::random(),
                        Car {
                            manufacturer: "Ford".to_string(),
                            name: "Raptor".to_string(),
                        },
                    ),
                    (
                        NonFungibleId::random(),
                        Car {
                            manufacturer: "Toyota".to_string(),
                            name: "Camry".to_string(),
                        },
                    ),
                    (
                        NonFungibleId::random(),
                        Car {
                            manufacturer: "Nissan".to_string(),
                            name: "Altima".to_string(),
                        },
                    ),
                ])
        }
    }
}
