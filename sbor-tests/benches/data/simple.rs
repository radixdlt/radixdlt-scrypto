use sbor::{Categorize, Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Categorize, Encode, Decode, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub enum SimpleEnum {
    Unit,
    Unamed(u32),
    Named { x: u32, y: u32 },
}

#[derive(Categorize, Encode, Decode, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct SimpleStruct {
    pub number: u64,
    pub string: String,
    pub vector1: Vec<u8>,
    pub vector2: Vec<u16>,
    pub enumeration: SimpleEnum,
}
