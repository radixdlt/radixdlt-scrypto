use crate::internal_prelude::*;

pub type ScryptoRawPayload<'a> = RawPayload<'a, ScryptoCustomExtension>;
pub type ScryptoUnvalidatedRawPayload<'a> = UnvalidatedRawPayload<'a, ScryptoCustomExtension>;
pub type ScryptoOwnedRawPayload = RawPayload<'static, ScryptoCustomExtension>;
pub type ScryptoUnvalidatedOwnedRawPayload = UnvalidatedRawPayload<'static, ScryptoCustomExtension>;
pub type ScryptoRawValue<'a> = RawValue<'a, ScryptoCustomExtension>;
pub type ScryptoUnvalidatedRawValue<'a> = UnvalidatedRawValue<'a, ScryptoCustomExtension>;
pub type ScryptoOwnedRawValue = RawValue<'static, ScryptoCustomExtension>;
pub type ScryptoUnvalidatedOwnedRawValue = UnvalidatedRawValue<'static, ScryptoCustomExtension>;
