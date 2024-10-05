use crate::internal_prelude::*;

/// This is the new compile/decompile target for saved transaction manifests.
///
/// ## Using AnyTransactionManifest
/// Typically you'll have a method `my_method` which takes a &impl ReadableManifest.
/// Ideally, we could have an apply method which lets you use this method trivially with
/// an [`AnyTransactionManifest`] - but this would require a function constraint of
/// `F: for<R: ReadableManifest> FnOnce<R, Output>` - which uses higher order type-based trait bounds
/// which don't exist yet (https://github.com/rust-lang/rust/issues/108185).
///
/// So instead, the convention is to also create an `my_method_any` with a switch statement in.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
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
    pub fn attempt_decode_from_arbitrary_payload(bytes: &[u8]) -> Result<Self, String> {
        // First, try to decode as AnyTransactionManifest
        if let Ok(any_manifest) = manifest_decode::<Self>(bytes) {
            return Ok(any_manifest);
        }

        // If that fails, try LegacyTransactionManifestV1
        if let Ok(legacy_v1_manifest) = manifest_decode::<LegacyTransactionManifestV1>(bytes) {
            return Ok(Self::V1(legacy_v1_manifest.into()));
        }

        // Finally, try as VersionedTransactionPayload
        if let Ok(any_transaction) = manifest_decode::<VersionedTransactionPayload>(bytes) {
            return Ok(match any_transaction {
                VersionedTransactionPayload::TransactionIntentV1(intent) => {
                    TransactionManifestV1::from_intent(&intent).into()
                }
                VersionedTransactionPayload::SignedTransactionIntentV1(signed_intent) => {
                    TransactionManifestV1::from_intent(&signed_intent.intent).into()
                }
                VersionedTransactionPayload::NotarizedTransactionV1(notarized) => {
                    TransactionManifestV1::from_intent(&notarized.signed_intent.intent).into()
                }
                VersionedTransactionPayload::SystemTransactionV1(system_transaction) => {
                    SystemTransactionManifestV1::from_transaction(&system_transaction).into()
                }
                VersionedTransactionPayload::TransactionIntentV2(intent) => {
                    TransactionManifestV2::from_intent_core(&intent.root_intent_core).into()
                }
                VersionedTransactionPayload::SignedTransactionIntentV2(signed_intent) => {
                    TransactionManifestV2::from_intent_core(
                        &signed_intent.transaction_intent.root_intent_core,
                    )
                    .into()
                }
                VersionedTransactionPayload::NotarizedTransactionV2(notarized) => {
                    TransactionManifestV2::from_intent_core(
                        &notarized
                            .signed_transaction_intent
                            .transaction_intent
                            .root_intent_core,
                    )
                    .into()
                }
                VersionedTransactionPayload::SubintentV2(subintent) => {
                    SubintentManifestV2::from_intent_core(&subintent.intent_core).into()
                }
                other_type => {
                    return Err(format!(
                        "Transaction type with discriminator {} not currently supported",
                        other_type.get_discriminator()
                    ))
                }
            });
        }

        Err(format!(
            "Cannot decode transaction manifest or transaction payload"
        ))
    }
}

impl ReadableManifestBase for AnyTransactionManifest {
    fn is_subintent(&self) -> bool {
        match self {
            AnyTransactionManifest::V1(m) => m.is_subintent(),
            AnyTransactionManifest::SystemV1(m) => m.is_subintent(),
            AnyTransactionManifest::V2(m) => m.is_subintent(),
            AnyTransactionManifest::SubintentV2(m) => m.is_subintent(),
        }
    }

    fn get_blobs<'a>(&'a self) -> impl Iterator<Item = (&'a Hash, &'a Vec<u8>)> {
        let iterator: Box<dyn Iterator<Item = (&'a Hash, &'a Vec<u8>)> + 'a> = match self {
            AnyTransactionManifest::V1(m) => Box::new(m.get_blobs()),
            AnyTransactionManifest::SystemV1(m) => Box::new(m.get_blobs()),
            AnyTransactionManifest::V2(m) => Box::new(m.get_blobs()),
            AnyTransactionManifest::SubintentV2(m) => Box::new(m.get_blobs()),
        };
        iterator
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        match self {
            AnyTransactionManifest::V1(m) => m.get_known_object_names_ref(),
            AnyTransactionManifest::SystemV1(m) => m.get_known_object_names_ref(),
            AnyTransactionManifest::V2(m) => m.get_known_object_names_ref(),
            AnyTransactionManifest::SubintentV2(m) => m.get_known_object_names_ref(),
        }
    }

    fn get_preallocated_addresses(&self) -> &[PreAllocatedAddress] {
        match self {
            AnyTransactionManifest::V1(m) => m.get_preallocated_addresses(),
            AnyTransactionManifest::SystemV1(m) => m.get_preallocated_addresses(),
            AnyTransactionManifest::V2(m) => m.get_preallocated_addresses(),
            AnyTransactionManifest::SubintentV2(m) => m.get_preallocated_addresses(),
        }
    }

    fn get_child_subintents(&self) -> &[ChildSubintent] {
        match self {
            AnyTransactionManifest::V1(m) => m.get_child_subintents(),
            AnyTransactionManifest::SystemV1(m) => m.get_child_subintents(),
            AnyTransactionManifest::V2(m) => m.get_child_subintents(),
            AnyTransactionManifest::SubintentV2(m) => m.get_child_subintents(),
        }
    }
}

impl ReadableManifest for AnyTransactionManifest {
    fn iter_instruction_effects(&self) -> impl Iterator<Item = ManifestInstructionEffect> {
        let iterator: Box<dyn Iterator<Item = ManifestInstructionEffect>> = match self {
            AnyTransactionManifest::V1(m) => Box::new(m.iter_instruction_effects()),
            AnyTransactionManifest::SystemV1(m) => Box::new(m.iter_instruction_effects()),
            AnyTransactionManifest::V2(m) => Box::new(m.iter_instruction_effects()),
            AnyTransactionManifest::SubintentV2(m) => Box::new(m.iter_instruction_effects()),
        };
        iterator
    }

    fn iter_cloned_instructions(&self) -> impl Iterator<Item = AnyInstruction> {
        let iterator: Box<dyn Iterator<Item = AnyInstruction>> = match self {
            AnyTransactionManifest::V1(m) => Box::new(m.iter_cloned_instructions()),
            AnyTransactionManifest::SystemV1(m) => Box::new(m.iter_cloned_instructions()),
            AnyTransactionManifest::V2(m) => Box::new(m.iter_cloned_instructions()),
            AnyTransactionManifest::SubintentV2(m) => Box::new(m.iter_cloned_instructions()),
        };
        iterator
    }

    fn instruction_count(&self) -> usize {
        match self {
            AnyTransactionManifest::V1(m) => m.instruction_count(),
            AnyTransactionManifest::SystemV1(m) => m.instruction_count(),
            AnyTransactionManifest::V2(m) => m.instruction_count(),
            AnyTransactionManifest::SubintentV2(m) => m.instruction_count(),
        }
    }

    fn instruction_effect(&self, index: usize) -> ManifestInstructionEffect {
        match self {
            AnyTransactionManifest::V1(m) => m.instruction_effect(index),
            AnyTransactionManifest::SystemV1(m) => m.instruction_effect(index),
            AnyTransactionManifest::V2(m) => m.instruction_effect(index),
            AnyTransactionManifest::SubintentV2(m) => m.instruction_effect(index),
        }
    }
}

pub enum ManifestKind {
    V1,
    SystemV1,
    V2,
    SubintentV2,
}

impl ManifestKind {
    const LATEST_SYSTEM: Self = Self::SystemV1;
    const LATEST_TRANSACTION: Self = Self::V2;
    const LATEST_SUBINTENT: Self = Self::SubintentV2;

    pub fn parse_or_latest(arg: Option<&str>) -> Result<Self, String> {
        match arg {
            Some(kind) => kind.try_into(),
            None => Ok(Self::LATEST_TRANSACTION),
        }
    }
}

impl TryFrom<&str> for ManifestKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let kind = match value.to_ascii_lowercase().as_str() {
            "v1" => Self::V1,
            "systemv1" => Self::SystemV1,
            "v2" => Self::V2,
            "subintentv2" => Self::SubintentV2,
            "system" => Self::LATEST_SYSTEM,
            "transaction" => Self::LATEST_TRANSACTION,
            "subintent" => Self::LATEST_SUBINTENT,
            _ => {
                return Err(format!(
                    "Manifest kind not recognized. Try one of: V1 | SystemV1 | V2 | SubintentV2"
                ))
            }
        };
        Ok(kind)
    }
}
