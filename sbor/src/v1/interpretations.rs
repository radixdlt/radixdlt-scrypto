pub struct DefaultInterpretations {
}

/// These are the default interpretations in base-Sbor
/// Specific variants may introduce more
impl DefaultInterpretations {
    // RESERVED
    /// A placeholder interpretation meaning the type / codec doesn't have one fixed
    /// interpretation. This value should never actually be seen in a payload.
    /// It would be used for the codecs of SBOR Value or smart pointers.
    pub const NOT_FIXED: u8 = 0x00;

    // MISC - Typically raw bytes
    pub const BOOLEAN: u8 = 0x01;
    pub const UTF8_STRING: u8 = 0x03;
    pub const UTF8_STRING_DISCRIMINATOR: u8 = 0x04;
    pub const SBOR_ANY: u8 = 0x04;
    pub const PLAIN_RAW_BYTES: u8 = 0x05;

    // UNSIGNED INTEGERS
    pub const U8: u8 = 0x10;
    pub const U16: u8 = 0x11;
    pub const U32: u8 = 0x12;
    pub const U64: u8 = 0x13;
    pub const U128: u8 = 0x14;
    pub const U256: u8 = 0x15;
    pub const USIZE: u8 = 0x1a;

    // SIGNED INTEGERS
    pub const I8: u8 = 0x20;
    pub const I16: u8 = 0x21;
    pub const I32: u8 = 0x22;
    pub const I64: u8 = 0x23;
    pub const I128: u8 = 0x24;
    pub const I256: u8 = 0x25;
    pub const ISIZE: u8 = 0x1b;

    // PRODUCT TYPE INTERPRETATIONS
    pub const UNIT: u8 = 0x30;
    pub const TUPLE: u8 = 0x31;
    pub const STRUCT: u8 = 0x32;
    pub const ENUM_VARIANT_UNIT: u8 = 0x33;
    pub const ENUM_VARIANT_TUPLE: u8 = 0x34;
    pub const ENUM_VARIANT_STRUCT: u8 = 0x35;

    // SUM TYPES
    pub const ENUM: u8 = 0x40;
    pub const RESULT: u8 = 0x41;
    pub const OPTION: u8 = 0x42;

    // LIST TYPES
    pub const NORMAL_LIST: u8 = 0x50;
    pub const FIXED_LENGTH_ARRAY: u8 = 0x52;
    /// The map defines no particular ordering of values
    pub const UNORDERED_SET: u8 = 0x5a;
    /// The map defines a particular ordering of keys (eg insertion order), respected by the serialization
    pub const ORDERED_SET: u8 = 0x5b;
    /// The map denotes that the keys are sorted by some ordering on the value space
    pub const SORTED_SET: u8 = 0x5c;

    // MAP TYPES
    /// The map defines no particular ordering of keys
    pub const UNORDERED_MAP: u8 = 0x6a;
    /// The map defines a particular ordering of keys (eg insertion order), respected by the serialization
    pub const ORDERED_MAP: u8 = 0x6b;
    /// The map denotes that the keys are sorted by some ordering on the key space
    pub const SORTED_MAP: u8 = 0x6c;
}