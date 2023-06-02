use scrypto::prelude::*;

#[blueprint]
mod preallocated_call {
    const PREALLOCATED: Global<Preallocated> = global_component!(Preallocated, "component_sim1cqqqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgp55w6zv");

    extern_blueprint!(
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
    const SOME_RESOURCE: ResourceManager = resource_manager!("resource_sim1t5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgpvd0xc6");

    struct SomeResource {}

    impl SomeResource {
        pub fn call_some_resource_total_supply() -> Decimal {
            SOME_RESOURCE.total_supply()
        }
    }
}
