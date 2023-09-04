use owned::*;
use scrypto::prelude::*;

#[blueprint]
#[events(GlobalBpEvent)]
mod global {
    pub struct GlobalBp(Owned<OwnedBp>);

    impl GlobalBp {
        pub fn new() -> Global<GlobalBp> {
            let this = Self(OwnedBp::new())
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            this.emit_event();
            this
        }

        pub fn emit_event(&self) {
            self.0.emit_event();
            Runtime::emit_event(GlobalBpEvent)
        }
    }
}

#[blueprint]
#[events(OwnedBpEvent)]
mod owned {
    pub struct OwnedBp {}

    impl OwnedBp {
        pub fn new() -> Owned<OwnedBp> {
            Self {}.instantiate()
        }

        pub fn emit_event(&self) {
            Runtime::emit_event(OwnedBpEvent)
        }
    }
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct OwnedBpEvent;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct GlobalBpEvent;
