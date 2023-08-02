use scrypto::prelude::*;

#[blueprint]
mod validator_access {
    struct ValidatorAccess {}

    impl ValidatorAccess {
        pub fn accepts_delegated_stake(validator: Global<Validator>) -> bool {
            validator.accepts_delegated_stake()
        }

        pub fn total_stake_xrd_amount(validator: Global<Validator>) -> Decimal {
            validator.total_stake_xrd_amount()
        }
    }
}
