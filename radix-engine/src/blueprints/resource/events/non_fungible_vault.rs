use crate::types::*;

macro_rules! define_events {
    ($($name: ident),* $(,)?) => {
        $(
            #[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
            pub struct $name {
                pub ids: BTreeSet<NonFungibleLocalId>,
            }

            impl $name {
                pub fn new(ids: BTreeSet<NonFungibleLocalId>) -> Self {
                    Self { ids }
                }
            }
        )*
    };
}
define_events! {
    WithdrawEvent,
    DepositEvent,
    RecallEvent
}
