use radix_engine_common::data::scrypto::{ScryptoDecode, ScryptoDescribe, ScryptoEncode};

pub trait ScryptoEvent
where
    Self: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
{
    const EVENT_NAME: &'static str;
}
