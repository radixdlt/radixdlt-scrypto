use scrypto::prelude::*;

#[blueprint]
mod tuple_return {
    struct TupleReturn;

    impl TupleReturn {
        pub fn instantiate() -> (Global<TupleReturn>, u64) {
            (
                Self.instantiate()
                    .prepare_to_globalize(OwnerRole::None)
                    .globalize(),
                18,
            )
        }
    }
}
