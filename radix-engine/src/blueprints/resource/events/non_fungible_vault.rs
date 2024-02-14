use crate::internal_prelude::*;

macro_rules! define_events {
    ($($name: ident),* $(,)?) => {
        $(
            #[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
            pub struct $name {
                pub ids: IndexSet<NonFungibleLocalId>,
            }

            impl $name {
                pub fn new(ids: IndexSet<NonFungibleLocalId>) -> Self {
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
