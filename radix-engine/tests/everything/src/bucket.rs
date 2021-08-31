use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct BucketTest;

    impl BucketTest {

        pub fn combine() -> Tokens {
            let resource = create_mutable_tokens("b1", Context::package_address());
            let tokens1 = mint_tokens(resource, 50);
            let tokens2 = mint_tokens(resource, 50);

            tokens1.put(tokens2);
            tokens1
        }

        pub fn split()  -> (Tokens, Tokens) {
            let resource = create_mutable_tokens("b2", Context::package_address());
            let tokens1 = mint_tokens(resource, 100);
            let tokens2 = tokens1.take(U256::from(5));
            (tokens1, tokens2)
        }

        pub fn borrow() -> Tokens {
            let resource = create_mutable_tokens("b3", Context::package_address());
            let tokens = mint_tokens(resource, 100);
            let reference = tokens.borrow();
            reference.destroy();
            tokens
        }

        pub fn query() -> (U256, Address, Tokens) {
            let resource = create_mutable_tokens("b4", Context::package_address());
            let tokens = mint_tokens(resource, 100);
            (tokens.amount(), tokens.resource(), tokens)
        }
    }
}
