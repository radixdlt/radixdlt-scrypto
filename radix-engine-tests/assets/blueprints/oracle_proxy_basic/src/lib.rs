use scrypto::prelude::*;

#[blueprint]
mod proxy {
    const ORACLE_PACKAGE_ADDRESS: PackageAddress = PackageAddress::new_or_panic([
        13, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
    ]);

    extern_blueprint!(
        ORACLE_PACKAGE_ADDRESS,
        Oracle {
            fn instantiate_owned() -> Owned<Oracle>;
            fn instantiate_global() -> Global<Oracle>;
            fn get_oracle_info(&self) -> String;
            fn set_price(&mut self, base: ResourceAddress, quote: ResourceAddress, price: Decimal);
            fn get_price(&self, base: ResourceAddress, quote: ResourceAddress) -> Option<Decimal>;
        }
    );

    struct OracleProxy {
        // Define what resources and data will be managed by Proxy components
        component_address: Option<Global<Oracle>>,
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
        pub fn set_component_address(&mut self, address: Global<Oracle>) {
            info!("Set component_address to {:?}", address);
            self.component_address = Some(address);
        }

        pub fn proxy_get_oracle_info(&self) -> String {
            let oracle_address = self.component_address.unwrap();
            oracle_address.get_oracle_info()
        }

        pub fn proxy_set_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
            price: Decimal,
        ) {
            let mut oracle_address = self.component_address.unwrap();
            oracle_address.set_price(base, quote, price);
        }

        pub fn proxy_get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> Option<Decimal> {
            let oracle_address = self.component_address.unwrap();
            oracle_address.get_price(base, quote)
        }
    }
}
