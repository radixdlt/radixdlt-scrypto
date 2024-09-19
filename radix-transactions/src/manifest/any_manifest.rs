use crate::internal_prelude::*;

/// ## Using AnyTransactionManifest
/// Typically you'll have a method `my_method` which takes a &impl ReadableManifest.
/// Ideally, we could have an apply method which lets you use this method trivially with
/// an [`AnyTransactionManifest`] - but this would require a function constraint of
/// `F: for<R: ReadableManifest> FnOnce<R, Output>` - which uses higher order type-based trait bounds
/// which don't exist yet (https://github.com/rust-lang/rust/issues/108185).
///
/// So instead, the convention is to also create an `my_method_any` with a switch statement in.
pub enum AnyTransactionManifest {
    V1(TransactionManifestV1),
    SystemV1(SystemTransactionManifestV1),
    V2(TransactionManifestV2),
    SubintentV2(SubintentManifestV2),
}

impl From<TransactionManifestV1> for AnyTransactionManifest {
    fn from(value: TransactionManifestV1) -> Self {
        Self::V1(value)
    }
}

impl From<SystemTransactionManifestV1> for AnyTransactionManifest {
    fn from(value: SystemTransactionManifestV1) -> Self {
        Self::SystemV1(value)
    }
}

impl From<TransactionManifestV2> for AnyTransactionManifest {
    fn from(value: TransactionManifestV2) -> Self {
        Self::V2(value)
    }
}

impl From<SubintentManifestV2> for AnyTransactionManifest {
    fn from(value: SubintentManifestV2) -> Self {
        Self::SubintentV2(value)
    }
}

impl AnyTransactionManifest {
    pub fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        match self {
            AnyTransactionManifest::V1(m) => m.get_blobs(),
            AnyTransactionManifest::SystemV1(m) => m.get_blobs(),
            AnyTransactionManifest::V2(m) => m.get_blobs(),
            AnyTransactionManifest::SubintentV2(m) => m.get_blobs(),
        }
    }
}
