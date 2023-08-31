use scrypto::prelude::*;

#[blueprint]
mod cast_test {
    struct CastTest {}

    impl CastTest {
        pub fn cast_to_validator(address: ComponentAddress) {
            let _validator: Global<Validator> = Global::from(address);
        }

        pub fn cast_to_any(address: ComponentAddress) {
            let _any_component: Global<AnyComponent> = Global::from(address);
        }
    }
}
