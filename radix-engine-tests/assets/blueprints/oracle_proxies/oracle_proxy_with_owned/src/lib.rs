use scrypto::prelude::*;

#[blueprint]
mod proxy {
    enable_method_auth! {
        roles {
            proxy_manager_auth => updatable_by: [];
        },
        methods {
            initialize_oracle => restrict_to: [proxy_manager_auth, OWNER];
            set_price => restrict_to: [proxy_manager_auth, OWNER];
            get_oracle_info => PUBLIC;
            get_price => PUBLIC;
        }
    }

    struct OracleProxy {
        // Vector used here, because it is not possible to drop Owned value.
        // Only the last added address will be used, previous addresses will be kept, despite not being used.
        oracle_owned_address: Vec<Owned<AnyComponent>>,
    }

    // This example demostrate a proxy working with:
    // - Oracle as an owned component (instantiated by proxy)
    //   Proxy can call any method from owned Oracle component
    impl OracleProxy {
        pub fn instantiate_and_globalize(
            owner_badge: NonFungibleGlobalId,
            manager_badge: NonFungibleGlobalId,
        ) -> Global<OracleProxy> {
            let owner_role = OwnerRole::Fixed(rule!(require(owner_badge)));
            let manager_rule = rule!(require(manager_badge));

            Self {
                oracle_owned_address: vec![],
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                proxy_manager_auth => manager_rule;
            })
            .globalize()
        }

        // Instantiate Oracle at given package address
        pub fn initialize_oracle(&mut self, oracle_package_address: PackageAddress) {
            info!("Instantiate oracla at address {:?}", oracle_package_address);
            let result = ScryptoVmV1Api::blueprint_call(
                oracle_package_address,
                "Oracle",
                "instantiate_owned",
                scrypto_args!(),
            );
            self.oracle_owned_address
                .push(scrypto_decode(&result).unwrap());
        }

        pub fn get_oracle_info(&self) -> String {
            let result = ScryptoVmV1Api::object_call(
                self.oracle_owned_address
                    .last()
                    .expect("Oracle not initialized")
                    .handle()
                    .as_node_id(),
                "get_oracle_info",
                scrypto_args!(),
            );

            scrypto_decode(&result).unwrap()
        }

        pub fn get_price(&self, base: ResourceAddress, quote: ResourceAddress) -> Option<Decimal> {
            let result = ScryptoVmV1Api::object_call(
                self.oracle_owned_address
                    .last()
                    .expect("Oracle not initialized")
                    .handle()
                    .as_node_id(),
                "get_price",
                scrypto_args!(base, quote),
            );
            scrypto_decode(&result).unwrap()
        }

        pub fn set_price(&self, base: ResourceAddress, quote: ResourceAddress, price: Decimal) {
            ScryptoVmV1Api::object_call(
                self.oracle_owned_address
                    .last()
                    .expect("Oracle not initialized")
                    .handle()
                    .as_node_id(),
                "set_price",
                scrypto_args!(base, quote, price),
            );
        }
    }
}
