use scrypto::prelude::*;

#[blueprint]
mod royalty_edge_cases {
    enable_package_royalties! {
        // We manipulate the value of this manually in tests by modifying the [`PackageDefinition`]
        instantiate => Free;
        func => Free;
        method => Free;
    }

    struct RoyaltyEdgeCases;

    impl RoyaltyEdgeCases {
        pub fn instantiate(royalty_amount: RoyaltyAmount) -> Global<RoyaltyEdgeCases> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updatable(AccessRule::AllowAll))
                .enable_component_royalties(component_royalties! {
                    init {
                        method => royalty_amount, updatable;
                    }
                })
                .globalize()
        }

        pub fn func() {}
        pub fn method(&self) {}
    }
}
