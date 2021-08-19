use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct BucketTest;

    impl BucketTest {

        pub fn combine() -> Tokens {
            let resource = create_mutable_tokens("b1", Context::blueprint_address());
            let mut a = mint_tokens(resource, 50);
            let b = mint_tokens(resource, 50);

            a.put(b);
            a
        }

        pub fn split()  -> (Tokens, Tokens) {
            let resource = create_mutable_tokens("b2", Context::blueprint_address());
            let mut a = mint_tokens(resource, 100);
            let b = a.take(U256::from(5));
            (a, b)
        }

        pub fn borrow() -> Tokens {
            let resource = create_mutable_tokens("b3", Context::blueprint_address());
            let a = mint_tokens(resource, 100);
            let r = a.borrow();
            r.destroy();
            a
        }

        pub fn query() -> (U256, Address, Tokens) {
            let resource = create_mutable_tokens("b4", Context::blueprint_address());
            let a = mint_tokens(resource, 100);
            (a.amount(), a.resource(), a)
        }
    }
}
