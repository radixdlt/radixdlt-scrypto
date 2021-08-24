use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::*;

blueprint! {
    struct MoveTest;

    impl MoveTest {

        pub fn receive_bucket(&self, t: Tokens) {
            info!("Received bucket: address = {}, amount = {}", t.resource(), t.amount());
            Account::from(Context::package_address()).deposit_tokens(t);
        }

        pub fn receive_reference(&self, t: TokensRef) {
            info!("Received reference: address = {}, amount = {}", t.resource(), t.amount());
            t.destroy();
        }

        pub fn move_bucket() {
            let resource =  create_mutable_tokens("m1", Context::package_address());
            let tokens =  mint_tokens(resource, 100);
            let component = MoveTest {}.instantiate();

            component.call::<()>("receive_bucket", args!(tokens));
        }

        pub fn move_reference() {
            let resource =  create_mutable_tokens("m2", Context::package_address());
            let tokens =  mint_tokens(resource, 100);
            let component = MoveTest {}.instantiate();

            component.call::<()>("receive_reference", args!(tokens.borrow()));

            // The sender still owns the tokens
            Account::from(Context::package_address()).deposit_tokens(tokens);
        }
    }
}
