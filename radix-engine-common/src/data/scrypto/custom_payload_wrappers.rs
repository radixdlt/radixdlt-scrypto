use super::*;
use sbor::*;

pub type ScryptoRawPayload<'a> = RawPayload<'a, ScryptoCustomTypeExtension>;
pub type ScryptoRawValue<'a> = RawValue<'a, ScryptoCustomTypeExtension>;
