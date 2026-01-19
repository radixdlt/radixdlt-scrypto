use scrypto::prelude::*;

#[blueprint]
mod tuple_return {
    struct TupleReturn;

    impl TupleReturn {
        pub fn instantiate() -> (Global<TupleReturn>, u64) {
            // This blueprint is also used for tests that check the
            // size with and without tracing logs.
            trace!("Trace message for testing purposes");
            (
                Self.instantiate()
                    .prepare_to_globalize(OwnerRole::None)
                    .globalize(),
                18,
            )
        }
    }
}
