use radix_engine_interface::math::*;
use radix_engine_interface::*;

#[derive(NonFungibleData, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct TestStruct {
    pub a: u32,
    #[legacy_skip]
    #[sbor(skip)]
    pub b: String,
    pub c: Decimal,
}

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub enum TestEnum {
    A { named: String },
    B(u32, u8, Decimal),
    C,
}
