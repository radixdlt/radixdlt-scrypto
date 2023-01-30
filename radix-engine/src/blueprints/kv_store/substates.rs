use crate::types::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode, PartialEq, Eq)]
pub enum KeyValueStoreEntrySubstate {
    Some(ScryptoValue, ScryptoValue),
    None,
}
