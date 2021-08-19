use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::*;

component! {
    struct MoveTest;

    impl MoveTest {

        pub fn receive_bucket(&self, t: Tokens) {
            info!("Received bucket: address = {}, amount = {}", t.resource(), t.amount());
            Account::from(Context::blueprint_address()).deposit_tokens(t);
        }

        pub fn receive_reference(&self, t: TokensRef) {
            info!("Received reference: address = {}, amount = {}", t.resource(), t.amount());
            t.destroy();
        }

        pub fn move_bucket() {
            let resource =  create_mutable_tokens("m1", Context::blueprint_address());
            let tokens =  mint_tokens(resource, 100);
            let component = Component::new("MoveTest", MoveTest {});

            call!((), "MoveTest", "receive_bucket", component.address(), tokens);
        }

        pub fn move_reference() {
            let resource =  create_mutable_tokens("m2", Context::blueprint_address());
            let tokens =  mint_tokens(resource, 100);
            let component = Component::new("MoveTest", MoveTest {});

            call!((), "MoveTest", "receive_reference", component.address(), tokens.borrow());

            // I still own the tokens
            Account::from(Context::blueprint_address()).deposit_tokens(tokens);
        }
    }
}
