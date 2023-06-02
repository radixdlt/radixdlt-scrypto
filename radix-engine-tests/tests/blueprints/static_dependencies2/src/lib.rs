use scrypto::prelude::*;

external_component! {
    Preallocated {
        fn get_secret(&self) -> String;
    }
}

#[blueprint]
mod preallocated_call {
    const PREALLOCATED: ComponentAddress = ComponentAddress::new_or_panic([
        192, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
    ]);

    struct PreallocatedCall {}

    impl PreallocatedCall {
        pub fn call_preallocated() -> String {
            let preallocated: Global<Preallocated> = PREALLOCATED.into();
            preallocated.get_secret()
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
