use scrypto::prelude::*;

#[blueprint]
mod cast_test {
    struct CastTest {}

    impl CastTest {
        pub fn cast_to_validator(address: ComponentAddress) -> Result<(), ComponentCastError> {
            let _validator: Global<Validator> = Global::try_from(address)?;
            Ok(())
        }
    }
}
