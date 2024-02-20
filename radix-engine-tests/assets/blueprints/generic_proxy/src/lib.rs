use scrypto::prelude::*;

#[blueprint]
mod proxy {
    struct GenericProxy {
        // Define what resources and data will be managed by Proxy components
        component_address: Option<Global<AnyComponent>>,
    }

    impl GenericProxy {
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate_proxy() -> Global<GenericProxy> {
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

        pub fn proxy_call(&self, method_name: String, args: ScryptoValue) -> ScryptoValue {
            let oracle_address = self.component_address.unwrap();
            let args = scrypto_encode(&args).unwrap();

            let bytes = ScryptoVmV1Api::object_call(
                oracle_address.address().as_node_id(),
                &method_name,
                args,
            );
            scrypto_decode(&bytes).unwrap()
        }
    }
}
