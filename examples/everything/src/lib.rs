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
                bucket: FungibleBucket
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
                    some_role => rule!(require(XRD));
                    some_role_updater => rule!(require(SOME_RESOURCE.address()));
                })
                .enable_component_royalties(component_royalties! {
                    roles {
                        royalty_setter => rule!(allow_all);
                        royalty_setter_updater => OWNER;
                        royalty_locker => OWNER;
                        royalty_locker_updater => rule!(deny_all);
                        royalty_claimer => OWNER;
                        royalty_claimer_updater => rule!(deny_all);
                    },
                    init {
                        public_method => Xrd(1.into()), updatable;
                        protected_method => Free, locked;
                    }
                })
                .metadata(metadata! {
                    roles {
                        metadata_locker => rule!(allow_all);
                        metadata_locker_updater => rule!(allow_all);
                        metadata_setter => OWNER;
                        metadata_setter_updater => rule!(deny_all);
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

        pub fn public_method(&self) -> NonFungibleResourceManager {
            ResourceBuilder::new_ruid_non_fungible::<TestNFData>(OwnerRole::None)
                .mint_roles(mint_roles! {
                    minter => rule!(allow_all);
                    minter_updater => rule!(allow_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(allow_all);
                })
                .freeze_roles(freeze_roles! {
                    freezer => rule!(allow_all);
                    freezer_updater => rule!(allow_all);
                })
                .recall_roles(recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(allow_all);
                })
                .withdraw_roles(withdraw_roles! {
                    withdrawer => rule!(allow_all);
                    withdrawer_updater => rule!(allow_all);
                })
                .deposit_roles(deposit_roles! {
                    depositor => rule!(allow_all);
                    depositor_updater => rule!(allow_all);
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater => rule!(allow_all);
                    non_fungible_data_updater_updater => rule!(allow_all);
                })
                .metadata(metadata! {
                    init {
                        "name" => "Super Admin Badge".to_string(), locked;
                    }
                })
                .create_with_no_initial_supply()
                .into()
        }

        pub fn protected_method(&self) {
            error!("This");
            warn!("is");
            info!("a");
            debug!("test");
            trace!("message");

            FAUCET.lock_fee(dec!("1.0"));
        }
    }
}
