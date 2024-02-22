use scrypto::prelude::*;

#[blueprint]
mod proxy {
    enable_method_auth! {
        methods {
            set_oracle_address => restrict_to: [OWNER];
            proxy_call => PUBLIC;
        }
    }

    struct GenericProxy {
        oracle_address: Option<Global<AnyComponent>>,
    }

    // This example assumes that:
    // - called component are instantiated as global component
    // - called methods of the component are not protected
    impl GenericProxy {
        pub fn instantiate_proxy(owner_role: OwnerRole) -> Global<GenericProxy> {
            Self {
                oracle_address: None,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        // Specify Oracle component address
        pub fn set_oracle_address(&mut self, address: Global<AnyComponent>) {
            info!("Set oracle address to {:?}", address);
            self.oracle_address = Some(address);
        }

        // This method allows to call any method from configured component by method name.
        // Method arguments must be encoded into ScryptoValue tuple of arguments.
        // It might be achieved by converting the arguments into ManifestValue, eg.
        //   - 2 arguments
        //   `let manifest_value = to_manifest_value(&(arg1, arg2))`
        //   - 1 argument
        //   `let manifest_value = to_manifest_value(&(arg1, ))`
        //   - no arguments
        //   `let manifest_value = to_manifest_value(&())`
        //
        //   So the full example could look like this
        //   ```
        //   let manifest = ManifestBuilder::new()
        //     .lock_fee_from_faucet()
        //     .call_method(
        //         proxy_component_address,
        //         "proxy_call",
        //         manifest_args!(
        //             "get_price",
        //             to_manifest_value(&("XRD".to_string(),)).unwrap()
        //         ),
        //     )
        //     .build();
        //  ```
        pub fn proxy_call(&self, method_name: String, args: ScryptoValue) -> ScryptoValue {
            let oracle_address = self.oracle_address.unwrap();
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
