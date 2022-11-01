/// A custom type is an application defined type with special semantics.
/// SborValues of a custom type must be encoded a `size + data`
pub const CUSTOM_TYPE_START: u8 = 0x80;

pub const OPTION_VARIANT_SOME: &str = "Some";
pub const OPTION_VARIANT_NONE: &str = "None";
pub const RESULT_VARIANT_OK: &str = "Ok";
pub const RESULT_VARIANT_ERR: &str = "Err";
