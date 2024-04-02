use crate::internal_prelude::*;

pub mod one_resource_pool {
    use super::*;

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct ContributionEvent {
        pub amount_of_resources_contributed: Decimal,
        pub pool_units_minted: Decimal,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct RedemptionEvent {
        pub pool_unit_tokens_redeemed: Decimal,
        pub redeemed_amount: Decimal,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct WithdrawEvent {
        pub amount: Decimal,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct DepositEvent {
        pub amount: Decimal,
    }
}

pub mod two_resource_pool {
    use super::*;

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct ContributionEvent {
        pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
        pub pool_units_minted: Decimal,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct RedemptionEvent {
        pub pool_unit_tokens_redeemed: Decimal,
        pub redeemed_resources: IndexMap<ResourceAddress, Decimal>,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct WithdrawEvent {
        pub resource_address: ResourceAddress,
        pub amount: Decimal,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct DepositEvent {
        pub resource_address: ResourceAddress,
        pub amount: Decimal,
    }
}

pub mod multi_resource_pool {
    use super::*;

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct ContributionEvent {
        pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
        pub pool_units_minted: Decimal,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct RedemptionEvent {
        pub pool_unit_tokens_redeemed: Decimal,
        pub redeemed_resources: IndexMap<ResourceAddress, Decimal>,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct WithdrawEvent {
        pub resource_address: ResourceAddress,
        pub amount: Decimal,
    }

    #[derive(ScryptoSbor, ScryptoEvent, Debug)]
    pub struct DepositEvent {
        pub resource_address: ResourceAddress,
        pub amount: Decimal,
    }
}
