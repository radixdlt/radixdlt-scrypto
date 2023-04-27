use super::*;
use sbor::*;

pub type ScryptoRawPayload<'a> = RawPayload<'a, ScryptoCustomTypeExtension>;
pub type ScryptoOwnedRawPayload = RawPayload<'static, ScryptoCustomTypeExtension>;
pub type ScryptoRawValue<'a> = RawValue<'a, ScryptoCustomTypeExtension>;
pub type ScryptoOwnedRawValue = RawValue<'static, ScryptoCustomTypeExtension>;
