pub struct Value {
    pub interpretation: u8,
    pub content: ValueContent,
}

pub enum ValueContent {
    RawBytes {
        bytes: Vec<u8>
    },
    Product {
        values: Vec<Value>
    },
    List {
        items: Vec<Value>
    },
    Map {
        entries: Vec<(Value, Value)>
    },
    Sum {
        discriminator: Discriminator,
        value: Box<Value>,
    },
}

pub enum Discriminator {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Any(Box<Value>),
}