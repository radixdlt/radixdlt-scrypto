use radix_engine_interface::*;

#[derive(ScryptoSbor, LegacyDescribe, NonFungibleData)]
pub struct TestStruct {
    pub a: u32,
    #[legacy_skip]
    #[sbor(skip)]
    pub b: String,
    // TODO fix me as part of the new schema integration
    // pub c: Decimal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub enum TestEnum {
    A { named: String },
    B(u32, u8),
    // C(Decimal),
}
