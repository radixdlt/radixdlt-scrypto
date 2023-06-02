use scrypto::prelude::*;

#[blueprint]
mod preallocated_call {
    const PREALLOCATED: Global<Preallocated> = at_address!("component_sim1cqqqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgp55w6zv");

    import_blueprint2!(
        "package_sim1p5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgpwgs6ac",
        Preallocated {
            fn get_secret(&self) -> String;
        }
    );

    struct PreallocatedCall {}

    impl PreallocatedCall {
        pub fn call_preallocated() -> String {
            PREALLOCATED.get_secret()
        }
    }
}

#[blueprint]
mod some_resource {
    const SOME_RESOURCE: ResourceManager =
        ResourceManager::from_address(ResourceAddress::new_or_panic([
            93, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1,
        ]));

    struct SomeResource {}

    impl SomeResource {
        pub fn call_some_resource_total_supply() -> Decimal {
            SOME_RESOURCE.total_supply()
        }
    }
}
