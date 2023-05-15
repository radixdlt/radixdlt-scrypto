use scrypto::prelude::*;

#[blueprint]
mod inner {
    struct Inner {}

    impl Inner {
        pub fn instantiate() -> Owned<Inner> {
            Self {}.instantiate()
        }

        pub fn receive_proof(&self, proof: Proof) {
            info!("{:?}", proof);
        }
    }
}

#[blueprint]
mod outer {
    use super::inner::{Inner, InnerComponent};

    struct Outer {
        inner: Owned<Inner>,
    }

    impl Outer {
        pub fn instantiate() -> Global<Outer> {
            let inner = Inner::instantiate();
            Self { inner }.instantiate().globalize()
        }

        pub fn pass_fungible_proof(&self, proof: Proof) {
            info!("Proof id is: {:?}", proof.0);
            self.inner.receive_proof(proof)
        }
    }
}
