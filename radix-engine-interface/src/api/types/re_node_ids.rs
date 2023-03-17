use radix_engine_common::data::scrypto::model::OBJECT_ID_LENGTH;

pub type LockHandle = u32;

pub type ObjectId = [u8; OBJECT_ID_LENGTH];
pub type KeyValueStoreId = [u8; OBJECT_ID_LENGTH];
