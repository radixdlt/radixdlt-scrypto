use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct ResourceTest {
        resource: Address,
        tokens: Tokens,
        secret: String,
    }

    impl ResourceTest {
        pub fn create_tokens() -> Address {
            let resource = Resource::new(
                "symbol",
                "name",
                "description",
                "url",
                "icon_url",
                Some(Context::address()),
                Some(U256::from(1000))
            );
            resource.into()
        }

        pub fn get_url(address: Address) -> String {
            let resource: Resource = address.into();
            resource.get_info().url
        }

        pub fn mint_tokens(address: Address) -> Tokens {
            let resource: Resource = address.into();
            resource.mint_tokens(U256::from(100))
        }
    }
}
