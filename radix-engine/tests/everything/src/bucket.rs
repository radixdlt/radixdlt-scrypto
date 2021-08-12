use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct BucketTest;

    impl BucketTest {

        pub fn combine() -> Tokens {
            let resource = Self::create_tokens("a".to_owned());
            let mut a = Self::mint_tokens(resource);
            let b = Self::mint_tokens(resource);

            a.put(b);
            a
        }

        pub fn split()  -> (Tokens, Tokens) {
            let resource = Self::create_tokens("b".to_owned());
            let mut a = Self::mint_tokens(resource);
            let b = a.take(U256::from(5));
            (a, b)
        }

        pub fn borrow() -> Tokens {
            let resource = Self::create_tokens("c".to_owned());
            let a = Self::mint_tokens(resource);
            let r = a.borrow();
            r.destroy();
            a
        }

        pub fn query() -> (U256, Address, Tokens) {
            let resource = Self::create_tokens("d".to_owned());
            let a = Self::mint_tokens(resource);
            (a.amount(), a.resource(), a)
        }

        pub fn create_tokens(symbol: String) -> Address {
            let resource = Resource::new(
                symbol.as_ref(),
                "name",
                "description",
                "url",
                "icon_url",
                Some(Context::address()),
                Some(U256::from(1000))
            );
            resource.into()
        }

        pub fn mint_tokens(address: Address) -> Tokens {
            let resource: Resource = address.into();
            resource.mint_tokens(U256::from(100))
        }
    }
}
