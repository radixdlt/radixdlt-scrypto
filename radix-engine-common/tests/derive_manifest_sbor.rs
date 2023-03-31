use radix_engine_common::math::*;
use radix_engine_common::*;

#[derive(ManifestSbor)]
pub struct TestStruct {
    pub a: u32,
    #[sbor(skip)]
    pub b: String,
    pub c: Decimal,
}

#[derive(ManifestSbor)]
pub enum TestEnum {
    A { named: String },
    B(u32, u8, Decimal),
    C,
}
