use crate::utils::*;
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
        pub fn create() -> Tokens {
           let resource = create_tokens("r1", 100);
           mint_tokens(resource, 100)
        }

        pub fn query() -> String {
            let resource: Resource = create_tokens("r2", 100).into();
            resource.get_info().url
        }
    }
}
