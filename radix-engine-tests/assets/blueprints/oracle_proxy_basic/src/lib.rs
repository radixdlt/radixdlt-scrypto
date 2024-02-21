use scrypto::prelude::*;

#[blueprint]
mod proxy {
    enable_method_auth! {
        methods {
            set_component_address => restrict_to: [OWNER];
            proxy_get_oracle_info => PUBLIC;
            proxy_get_price => PUBLIC;
        }
    }

    struct OracleProxy {
        component_address: Option<Global<AnyComponent>>,
    }

    // This example assumes that:
    // - called component are instantiated as global component
    // - called methods of the component are not protected
    impl OracleProxy {
        pub fn instantiate_proxy(owner_role: OwnerRole) -> Global<OracleProxy> {
            Self {
                component_address: None,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        // Specify Oracle component address
        pub fn set_component_address(&mut self, address: Global<AnyComponent>) {
            info!("Set component_address to {:?}", address);
            self.component_address = Some(address);
        }

        pub fn proxy_get_oracle_info(&self) -> String {
            let oracle_address = self.component_address.unwrap();

            let result = ScryptoVmV1Api::object_call(
                oracle_address.address().as_node_id(),
                "get_oracle_info",
                scrypto_args!(),
            );

            scrypto_decode(&result).unwrap()
        }

        pub fn proxy_get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> Option<Decimal> {
            let oracle_address = self.component_address.unwrap();

            let result = ScryptoVmV1Api::object_call(
                oracle_address.address().as_node_id(),
                "get_price",
                scrypto_args!(base, quote),
            );
            scrypto_decode(&result).unwrap()
        }
    }
}
