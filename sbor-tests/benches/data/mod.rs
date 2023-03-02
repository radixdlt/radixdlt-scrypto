use sbor::rust::prelude::*;
use sbor::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Sbor, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub enum SimpleEnum {
    Unit,
    Unnamed(u32),
    Named { x: u32, y: u32 },
}

#[derive(Debug, Clone, Sbor, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct SimpleStruct {
    pub number: u64,
    pub string: String,
    pub bytes: Vec<u8>,
    pub vector: Vec<u16>,
    pub enumeration: Vec<SimpleEnum>,
    pub map: BTreeMap<String, String>,
}

pub fn get_simple_dataset(repeat: usize) -> SimpleStruct {
    let mut data = SimpleStruct {
        number: 12345678901234567890,
        string: "dummy".repeat(repeat).to_owned(),
        bytes: vec![123u8; repeat],
        vector: vec![12345u16; repeat],
        enumeration: vec![
            SimpleEnum::Named {
                x: 1234567890,
                y: 1234567890,
            };
            repeat
        ],
        map: BTreeMap::new(),
    };

    for i in 0..repeat {
        data.map
            .insert(format!("Key_{}", i), format!("Value_{}", i));
    }

    data
}
