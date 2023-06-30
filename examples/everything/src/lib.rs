use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct TestNFData {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod everything {
    enable_function_auth! {
        new => rule!(deny_all);
        another_function => rule!(allow_all);
    }

    enable_method_auth! {
        roles {
            some_role => updatable_by: [SELF, OWNER, some_role_updater];
            some_role_updater => updatable_by: [];
        },
        methods {
            public_method => PUBLIC;
            protected_method => restrict_to: [some_role, OWNER];
        }
    }

    enable_package_royalties! {
        new => Xrd(2.into());
        another_function => Xrd(2.into());
        public_method => Free;
        protected_method => Free;
    }

    extern_blueprint!(
        "package_rdx1pkgxxxxxxxxxfaucetxxxxxxxxx000034355863xxxxxxxxxfaucet",
        Faucet as FiFi {
            fn new(
                address_reservation: GlobalAddressReservation,
                bucket: Bucket
            ) -> Global<FiFi>;

            fn lock_fee(&self, amount: Decimal);
        }
    );

    const FAUCET: Global<FiFi> = global_component!(
        FiFi,
        "component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh"
    );
    const SOME_RESOURCE: ResourceManager =
        resource_manager!("resource_sim1t5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqqyqszqgqqqqqqqgpvd0xc6");

    struct Everything {}

    impl Everything {
        pub fn new() -> Global<Everything> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles! {
                    some_role => rule!(require(XRD)), updatable;
                    some_role_updater => rule!(require(SOME_RESOURCE.address())), locked;
                })
                .enable_component_royalties(component_royalties! {
                    roles {
                        royalty_admin => rule!(allow_all), updatable;
                        royalty_admin_updater => OWNER, locked;
                    },
                    init {
                        public_method => Xrd(1.into()), updatable;
                        protected_method => Free, locked;
                    }
                })
                .metadata(metadata! {
                    roles {
                        metadata_locker => rule!(allow_all), locked;
                        metadata_locker_updater => rule!(allow_all), locked;
                        metadata_setter => OWNER, locked;
                        metadata_setter_updater => rule!(deny_all), locked;
                    },
                    init {
                        "some_key" => "string_value".to_string(), updatable;
                        "empty_locked" => EMPTY, locked;
                    }
                })
                .globalize()
        }

        pub fn another_function(faucet: Global<FiFi>) {
            let amount: Decimal = 10.into();
            faucet.lock_fee(amount);
        }

        pub fn public_method(&self) -> ResourceManager {
            ResourceBuilder::new_ruid_non_fungible::<TestNFData>(OwnerRole::None)
                .metadata(metadata! {
                    init {
                        "name" => "Super Admin Badge".to_string(), locked;
                    }
                })
                .mintable(rule!(allow_all), rule!(allow_all))
                .create_with_no_initial_supply()
        }

        pub fn protected_method(&self) {
            FAUCET.lock_fee(dec!("1.0"));
        }
    }
}
