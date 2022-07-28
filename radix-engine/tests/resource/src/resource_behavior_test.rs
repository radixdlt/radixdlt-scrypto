use scrypto::prelude::*;

blueprint! {
    struct ResourceBehaviorTest {}

    impl ResourceBehaviorTest {
        pub fn create_new_resource(
            resource_type: ResourceType,
            behavior_to_add: ResourceMethodAuthKey,
            behavior_access_rule: AccessRule,
            mutability_access_rule: Mutability,
        ) -> ResourceAddress {
            match resource_type {
                ResourceType::NonFungible => Self::create_new_non_fungible_resource(
                    behavior_to_add,
                    behavior_access_rule,
                    mutability_access_rule,
                ),
                ResourceType::Fungible { divisibility } => Self::create_new_fungible_resource(
                    divisibility,
                    behavior_to_add,
                    behavior_access_rule,
                    mutability_access_rule,
                ),
            }
        }

        pub fn create_new_fungible_resource(
            divisibility: u8,
            behavior_to_add: ResourceMethodAuthKey,
            behavior_access_rule: AccessRule,
            mutability_access_rule: Mutability,
        ) -> ResourceAddress {
            let mut fungible_resource_builder = ResourceBuilder::new_fungible();
            let fungible_resource_builder = fungible_resource_builder.divisibility(divisibility);
            let fungible_resource_builder =
                match behavior_to_add {
                    ResourceMethodAuthKey::Mint => fungible_resource_builder
                        .mintable(behavior_access_rule, mutability_access_rule),
                    ResourceMethodAuthKey::Burn => fungible_resource_builder
                        .burnable(behavior_access_rule, mutability_access_rule),
                    ResourceMethodAuthKey::Deposit => fungible_resource_builder
                        .restrict_deposit(behavior_access_rule, mutability_access_rule),
                    ResourceMethodAuthKey::Withdraw => fungible_resource_builder
                        .restrict_withdraw(behavior_access_rule, mutability_access_rule),
                    ResourceMethodAuthKey::UpdateMetadata => fungible_resource_builder
                        .updateable_metadata(behavior_access_rule, mutability_access_rule),
                    ResourceMethodAuthKey::UpdateNonFungibleData => {
                        panic!("Not supported on fungible tokens")
                    }
                };
            fungible_resource_builder.no_initial_supply()
        }

        pub fn create_new_non_fungible_resource(
            behavior_to_add: ResourceMethodAuthKey,
            behavior_access_rule: AccessRule,
            mutability_access_rule: Mutability,
        ) -> ResourceAddress {
            let mut non_fungible_resource_builder = ResourceBuilder::new_non_fungible();
            let non_fungible_resource_builder = match behavior_to_add {
                ResourceMethodAuthKey::Mint => non_fungible_resource_builder
                    .mintable(behavior_access_rule, mutability_access_rule),
                ResourceMethodAuthKey::Burn => non_fungible_resource_builder
                    .burnable(behavior_access_rule, mutability_access_rule),
                ResourceMethodAuthKey::Deposit => non_fungible_resource_builder
                    .restrict_deposit(behavior_access_rule, mutability_access_rule),
                ResourceMethodAuthKey::Withdraw => non_fungible_resource_builder
                    .restrict_withdraw(behavior_access_rule, mutability_access_rule),
                ResourceMethodAuthKey::UpdateMetadata => non_fungible_resource_builder
                    .updateable_metadata(behavior_access_rule, mutability_access_rule),
                ResourceMethodAuthKey::UpdateNonFungibleData => non_fungible_resource_builder
                    .updateable_non_fungible_data(behavior_access_rule, mutability_access_rule),
            };
            non_fungible_resource_builder.no_initial_supply()
        }

        pub fn create_and_check_resource_behavior(
            resource_type: ResourceType,
            behavior_to_add: ResourceMethodAuthKey,
            behavior_access_rule: AccessRule,
            mutability_access_rule: Mutability,

            check_for_behavior: ResourceMethodAuthKey,
        ) -> (bool, bool) {
            let resource_address = Self::create_new_resource(
                resource_type,
                behavior_to_add,
                behavior_access_rule,
                mutability_access_rule,
            );

            let resource_manager = borrow_resource_manager!(resource_address);
            match check_for_behavior {
                ResourceMethodAuthKey::Mint => (
                    resource_manager.is_mintable(),
                    resource_manager.is_mintable_locked(),
                ),
                ResourceMethodAuthKey::Burn => (
                    resource_manager.is_burnable(),
                    resource_manager.is_burnable_locked(),
                ),
                ResourceMethodAuthKey::Deposit => (
                    resource_manager.is_depositable(),
                    resource_manager.is_depositable_locked(),
                ),
                ResourceMethodAuthKey::Withdraw => (
                    resource_manager.is_withdrawable(),
                    resource_manager.is_withdrawable_locked(),
                ),
                ResourceMethodAuthKey::UpdateMetadata => (
                    resource_manager.is_updatable_metadata(),
                    resource_manager.is_updatable_metadata_locked(),
                ),
                ResourceMethodAuthKey::UpdateNonFungibleData => (
                    resource_manager.is_updatable_non_fungible_data(),
                    resource_manager.is_updatable_non_fungible_data_locked(),
                ),
            }
        }
    }
}
