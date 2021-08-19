use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct AccountTest;

    impl AccountTest {

        pub fn deposit_and_withdraw() -> Tokens {
            let resource =  create_mutable_tokens("a1", Context::blueprint_address());
            let tokens =  mint_tokens(resource, 100);

            let account = Account::from(Context::blueprint_address());
            account.deposit_tokens(tokens);
            account.withdraw_tokens(U256::from(10), resource)
        }
    }
}
