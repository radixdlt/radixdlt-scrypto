use scrypto::prelude::*;

#[blueprint]
mod inner {
    struct Inner {}

    impl Inner {
        pub fn instantiate() -> InnerComponent {
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
        inner: InnerComponent,
    }

    impl Outer {
        pub fn instantiate() -> Global<OuterComponent> {
            let inner = Inner::instantiate();
            Self { inner }
                .instantiate()
                .globalize_with_access_rules(MethodAuthorities::new(), AuthorityRules::new())
        }

        pub fn pass_fungible_proof(&self, proof: Proof) {
            info!("Proof id is: {:?}", proof.0);
            self.inner.receive_proof(proof)
        }
    }
}
