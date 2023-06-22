use crate::internal_prelude::*;

pub type ManifestRawPayload<'a> = RawPayload<'a, ManifestCustomExtension>;
pub type ManifestOwnedRawPayload = RawPayload<'static, ManifestCustomExtension>;
pub type ManifestRawValue<'a> = RawValue<'a, ManifestCustomExtension>;
pub type ManifestOwnedRawValue = RawValue<'static, ManifestCustomExtension>;
