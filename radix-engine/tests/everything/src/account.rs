use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct AccountTest;

    impl AccountTest {

        pub fn deposit_and_withdraw() -> Tokens {
            let resource = Self::create_tokens("a".to_owned());
            let tokens = Self::mint_tokens(resource);

            let account = Account::from(Context::address());
            account.deposit_tokens(tokens);
            account.withdraw_tokens(U256::from(10), resource)
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
