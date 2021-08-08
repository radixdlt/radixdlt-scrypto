// primitives
pub const TYPE_UNIT: u8 = 0x00;
pub const TYPE_BOOL: u8 = 0x01;
pub const TYPE_I8: u8 = 0x02;
pub const TYPE_I16: u8 = 0x03;
pub const TYPE_I32: u8 = 0x04;
pub const TYPE_I64: u8 = 0x05;
pub const TYPE_I128: u8 = 0x06;
pub const TYPE_U8: u8 = 0x07;
pub const TYPE_U16: u8 = 0x08;
pub const TYPE_U32: u8 = 0x09;
pub const TYPE_U64: u8 = 0x0a;
pub const TYPE_U128: u8 = 0x0b;
pub const TYPE_STRING: u8 = 0x0c;
// rust types
pub const TYPE_OPTION: u8 = 0x10;
pub const TYPE_BOX: u8 = 0x11;
pub const TYPE_ARRAY: u8 = 0x12;
pub const TYPE_TUPLE: u8 = 0x13;
pub const TYPE_STRUCT: u8 = 0x14;
pub const TYPE_ENUM: u8 = 0x15;
pub const TYPE_FIELDS_NAMED: u8 = 0x16;
pub const TYPE_FIELDS_UNNAMED: u8 = 0x17;
pub const TYPE_FIELDS_UNIT: u8 = 0x18;
// collections
pub const TYPE_VEC: u8 = 0x20;
pub const TYPE_TREE_SET: u8 = 0x21;
pub const TYPE_TREE_MAP: u8 = 0x22;
pub const TYPE_HASH_SET: u8 = 0x23;
pub const TYPE_HASH_MAP: u8 = 0x24;
