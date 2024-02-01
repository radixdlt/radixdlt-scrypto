use scrypto::prelude::*;

#[blueprint]
mod validator_access {
    struct ValidatorAccess {}

    impl ValidatorAccess {
        pub fn accepts_delegated_stake(mut validator: Global<Validator>) -> bool {
            validator.accepts_delegated_stake()
        }

        pub fn total_stake_xrd_amount(validator: Global<Validator>) -> Decimal {
            validator.total_stake_xrd_amount()
        }

        pub fn total_stake_unit_supply(validator: Global<Validator>) -> Decimal {
            validator.total_stake_unit_supply()
        }

        pub fn get_redemption_value(validator: Global<Validator>, amount: Decimal) -> Decimal {
            validator.get_redemption_value(amount)
        }
    }
}
