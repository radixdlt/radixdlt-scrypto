use scrypto::prelude::*;

#[blueprint]
mod preallocated_call {
    extern_blueprint!(
        "package_sim1p5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgpwgs6ac",
        Preallocated {
            fn get_secret(&self) -> String;
        }
    );

    struct PreallocatedCall {}

    impl PreallocatedCall {
        pub fn call_preallocated() -> String {
            let component = global_component!(
                Preallocated,
                "component_sim1cqqqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgp55w6zv"
            );
            component.get_secret()
        }
    }
}

#[blueprint]
mod some_resource {
    struct SomeResource {}

    impl SomeResource {
        pub fn call_some_resource_total_supply() -> Decimal {
            resource_manager!("resource_sim1t5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgpvd0xc6")
                .total_supply()
                .unwrap()
        }

        pub fn resource() -> ResourceManager {
            resource_manager!("resource_sim1t5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgpvd0xc6")
        }
    }
}

#[blueprint]
mod some_package {
    const SOME_PACKAGE: Package =
        package!("package_sim1p5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgpwgs6ac");

    struct SomePackage {}

    impl SomePackage {
        pub fn set_package_metadata() {
            SOME_PACKAGE.set_metadata("key", "value".to_string());
        }
    }
}
