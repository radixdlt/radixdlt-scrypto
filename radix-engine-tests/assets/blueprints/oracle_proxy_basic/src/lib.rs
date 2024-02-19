use scrypto::prelude::*;

#[blueprint]
mod proxy {
    struct OracleProxy {
        // Define what resources and data will be managed by Proxy components
        component_address: Option<Global<AnyComponent>>,
    }

    impl OracleProxy {
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate_proxy() -> Global<OracleProxy> {
            // Instantiate a Proxy component
            Self {
                component_address: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
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

        pub fn proxy_set_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
            price: Decimal,
        ) {
            let oracle_address = self.component_address.unwrap();

            let _result = ScryptoVmV1Api::object_call(
                oracle_address.address().as_node_id(),
                "set_price",
                scrypto_args!(base, quote, price),
            );
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
