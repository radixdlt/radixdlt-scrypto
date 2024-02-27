use scrypto::prelude::*;

#[blueprint]
mod proxy {
    enable_method_auth! {
        roles {
            proxy_manager_auth => updatable_by: [];
        },
        methods {
            set_component_address => restrict_to: [proxy_manager_auth, OWNER];
            call_method => PUBLIC;
        }
    }

    struct OracleGenericProxy {
        component_address: Option<Global<AnyComponent>>,
    }

    // This example assumes that:
    // - called component are instantiated as global component
    // - called methods of the component are not protected
    impl OracleGenericProxy {
        pub fn instantiate_and_globalize(
            owner_badge: NonFungibleGlobalId,
            manager_badge: NonFungibleGlobalId,
        ) -> Global<OracleGenericProxy> {
            let owner_role = OwnerRole::Fixed(rule!(require(owner_badge)));
            let manager_rule = rule!(require(manager_badge));

            Self {
                component_address: None,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                proxy_manager_auth => manager_rule;
            })
            .globalize()
        }

        // Specify Oracle component address
        pub fn set_component_address(&mut self, address: Global<AnyComponent>) {
            info!("Set component address to {:?}", address);
            self.component_address = Some(address);
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
        //         "call_method",
        //         manifest_args!(
        //             "get_price",
        //             to_manifest_value(&("XRD".to_string(),)).unwrap()
        //         ),
        //     )
        //     .build();
        //  ```
        pub fn call_method(&self, method_name: String, args: ScryptoValue) -> ScryptoValue {
            let args = scrypto_encode(&args).unwrap();

            let bytes = ScryptoVmV1Api::object_call(
                self.component_address
                    .expect("Component address not set")
                    .handle()
                    .as_node_id(),
                &method_name,
                args,
            );
            scrypto_decode(&bytes).unwrap()
        }
    }
}
