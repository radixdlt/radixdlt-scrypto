use crate::types::*;

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
    WithdrawEvent,
    DepositEvent,
    RecallEvent
}
