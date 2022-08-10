use scrypto::prelude::*;

blueprint!{
    struct Numbers {}

    impl Numbers {
        pub fn test_input(
            _: Decimal,
            _: PreciseDecimal,
            _: u8, _: u16, _: u32, _: u64, _: u128,
            _: i8, _: i16, _: i32, _: i64, _: i128,
            _: U8, _: U16, _: U32, _: U64, _: U128, _: U256, _: U384, _: U512,
            _: I8, _: I16, _: I32, _: I64, _: I128, _: I256, _: I384, _: I512,
        ) {
            info!("Call succeeded");
        }
    }
}