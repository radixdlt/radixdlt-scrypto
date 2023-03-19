use radix_engine_common::data::scrypto::{ScryptoDecode, ScryptoDescribe, ScryptoEncode};

pub trait ScryptoEvent
where
    Self: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
{
    fn event_name() -> &'static str;
}
