use inner::*;
use scrypto::prelude::*;

#[blueprint]
#[events(OuterEvent)]
mod outer {
    pub struct Outer(Owned<Inner>);

    impl Outer {
        pub fn new() -> Global<Outer> {
            let this = Self(Inner::new())
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            this.emit_event();
            this
        }

        pub fn emit_event(&self) {
            self.0.emit_event();
            Runtime::emit_event(OuterEvent)
        }
    }
}

#[blueprint]
#[events(InnerEvent)]
mod inner {
    pub struct Inner {}

    impl Inner {
        pub fn new() -> Owned<Inner> {
            Self {}.instantiate()
        }

        pub fn emit_event(&self) {
            Runtime::emit_event(InnerEvent)
        }
    }
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct InnerEvent;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct OuterEvent;
