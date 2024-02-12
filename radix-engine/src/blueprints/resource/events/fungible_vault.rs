use crate::internal_prelude::*;

macro_rules! define_events {
    ($($name: ident),* $(,)?) => {
        $(
            #[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
            pub struct $name {
                pub amount: Decimal,
            }

            impl $name {
                pub fn new(amount: Decimal) -> Self {
                    Self { amount }
                }
            }
        )*
    };
}
define_events! {
    LockFeeEvent,
    PayFeeEvent,
    WithdrawEvent,
    DepositEvent,
    RecallEvent
}
