use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::rust::vec::Vec;
use scrypto::*;

blueprint! {
    struct MoveTest {
        tokens: Vec<Tokens>
    }

    impl MoveTest {

        pub fn receive_bucket(&mut self, t: Tokens) {
            info!("Received bucket: address = {}, amount = {}", t.resource(), t.amount());
            self.tokens.push(t);
        }

        pub fn receive_reference(&self, t: TokensRef) {
            info!("Received reference: address = {}, amount = {}", t.resource(), t.amount());
            t.drop();
        }

        pub fn move_bucket() {
            let resource =  create_mutable_tokens("m1", Context::package_address());
            let tokens =  mint_tokens(resource, 100);
            let component: Component = MoveTest {
                tokens: Vec::new()
            }.instantiate().into();

            component.call::<()>("receive_bucket", args!(tokens));
        }

        pub fn move_reference() -> Tokens {
            let resource =  create_mutable_tokens("m2", Context::package_address());
            let tokens =  mint_tokens(resource, 100);
            let component: Component = MoveTest {
                tokens: Vec::new()
            }.instantiate().into();

            component.call::<()>("receive_reference", args!(tokens.borrow()));

            // The package still owns the tokens
            tokens
        }
    }
}
