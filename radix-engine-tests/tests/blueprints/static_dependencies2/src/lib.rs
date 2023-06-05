use scrypto::prelude::*;

// TODO: Change this to be a stub
#[blueprint]
mod preallocated {
    struct Preallocated {
        secret: String,
    }

    impl Preallocated {
        pub fn new(preallocated_address_bytes: [u8; 30], secret: String) -> Global<Preallocated> {
            Self { secret }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(ComponentAddress::new_or_panic(preallocated_address_bytes))
                .globalize()
        }

        pub fn get_secret(&self) -> String {
            self.secret.clone()
        }
    }
}

#[blueprint]
mod preallocated_call {
    use super::preallocated::*;

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
