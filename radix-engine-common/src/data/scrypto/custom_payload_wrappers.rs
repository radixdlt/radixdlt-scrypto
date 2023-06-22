use crate::internal_prelude::*;

pub type ScryptoRawPayload<'a> = RawPayload<'a, ScryptoCustomExtension>;
pub type ScryptoOwnedRawPayload = RawPayload<'static, ScryptoCustomExtension>;
pub type ScryptoRawValue<'a> = RawValue<'a, ScryptoCustomExtension>;
pub type ScryptoOwnedRawValue = RawValue<'static, ScryptoCustomExtension>;
