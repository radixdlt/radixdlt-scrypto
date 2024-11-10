use crate::internal_prelude::*;

/// This is the new compile/decompile target for saved transaction manifests,
/// and generally a type to support an unknown kind of manifest at runtime.
///
/// ## Using AnyManifest
/// Sometimes a method can take &impl ReadableManifest, which is preferred if possible.
///
/// Sometimes however a particular type is required for a method, needing an impl [`TypedReadableManifest`].
/// In which case, we can add a `XXX_any(...)` method which takes an [`AnyManifest`]
/// and then uses a `match` statement to delegate to the correct typed method.
///
/// Ideally, we could have an apply method which lets you use this method trivially with
/// an [`AnyManifest`] - but this would require a function constraint of
/// `F: for<R: ReadableManifest> FnOnce<R, Output>` - which uses higher order type-based trait bounds
/// which don't exist yet (https://github.com/rust-lang/rust/issues/108185).
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum AnyManifest {
    V1(TransactionManifestV1),
    SystemV1(SystemTransactionManifestV1),
    V2(TransactionManifestV2),
    SubintentV2(SubintentManifestV2),
}

impl From<TransactionManifestV1> for AnyManifest {
    fn from(value: TransactionManifestV1) -> Self {
        Self::V1(value)
    }
}

impl TryFrom<AnyManifest> for TransactionManifestV1 {
    type Error = ();

    fn try_from(value: AnyManifest) -> Result<Self, Self::Error> {
        match value {
            AnyManifest::V1(manifest) => Ok(manifest),
            _ => Err(()),
        }
    }
}

impl From<SystemTransactionManifestV1> for AnyManifest {
    fn from(value: SystemTransactionManifestV1) -> Self {
        Self::SystemV1(value)
    }
}

impl TryFrom<AnyManifest> for SystemTransactionManifestV1 {
    type Error = ();

    fn try_from(value: AnyManifest) -> Result<Self, Self::Error> {
        match value {
            AnyManifest::SystemV1(manifest) => Ok(manifest),
            _ => Err(()),
        }
    }
}

impl From<TransactionManifestV2> for AnyManifest {
    fn from(value: TransactionManifestV2) -> Self {
        Self::V2(value)
    }
}

impl TryFrom<AnyManifest> for TransactionManifestV2 {
    type Error = ();

    fn try_from(value: AnyManifest) -> Result<Self, Self::Error> {
        match value {
            AnyManifest::V2(manifest) => Ok(manifest),
            _ => Err(()),
        }
    }
}

impl From<SubintentManifestV2> for AnyManifest {
    fn from(value: SubintentManifestV2) -> Self {
        Self::SubintentV2(value)
    }
}

impl TryFrom<AnyManifest> for SubintentManifestV2 {
    type Error = ();

    fn try_from(value: AnyManifest) -> Result<Self, Self::Error> {
        match value {
            AnyManifest::SubintentV2(manifest) => Ok(manifest),
            _ => Err(()),
        }
    }
}

// It's not technically a conventional transaction payload, but let's reuse the macro
define_raw_transaction_payload!(RawManifest, TransactionPayloadKind::Other);

impl AnyManifest {
    pub fn to_raw(&self) -> Result<RawManifest, EncodeError> {
        Ok(RawManifest::from_vec(manifest_encode(self)?))
    }

    pub fn from_raw(raw: &RawManifest) -> Result<Self, DecodeError> {
        Ok(manifest_decode(raw.as_slice())?)
    }

    pub fn attempt_decode_from_arbitrary_payload(bytes: &[u8]) -> Result<Self, String> {
        // First, try to decode as AnyManifest
        if let Ok(any_manifest) = manifest_decode::<Self>(bytes) {
            return Ok(any_manifest);
        }

        // If that fails, try LegacyTransactionManifestV1
        if let Ok(legacy_v1_manifest) = manifest_decode::<LegacyTransactionManifestV1>(bytes) {
            return Ok(Self::V1(legacy_v1_manifest.into()));
        }

        // Finally, try as AnyTransaction
        if let Ok(any_transaction) = manifest_decode::<AnyTransaction>(bytes) {
            return Ok(match any_transaction {
                AnyTransaction::TransactionIntentV1(intent) => {
                    TransactionManifestV1::from_intent(&intent).into()
                }
                AnyTransaction::SignedTransactionIntentV1(signed_intent) => {
                    TransactionManifestV1::from_intent(&signed_intent.intent).into()
                }
                AnyTransaction::NotarizedTransactionV1(notarized) => {
                    TransactionManifestV1::from_intent(&notarized.signed_intent.intent).into()
                }
                AnyTransaction::SystemTransactionV1(system_transaction) => {
                    SystemTransactionManifestV1::from_transaction(&system_transaction).into()
                }
                AnyTransaction::TransactionIntentV2(intent) => {
                    TransactionManifestV2::from_intent_core(&intent.root_intent_core).into()
                }
                AnyTransaction::SignedTransactionIntentV2(signed_intent) => {
                    TransactionManifestV2::from_intent_core(
                        &signed_intent.transaction_intent.root_intent_core,
                    )
                    .into()
                }
                AnyTransaction::NotarizedTransactionV2(notarized) => {
                    TransactionManifestV2::from_intent_core(
                        &notarized
                            .signed_transaction_intent
                            .transaction_intent
                            .root_intent_core,
                    )
                    .into()
                }
                AnyTransaction::SubintentV2(subintent) => {
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

impl ReadableManifestBase for AnyManifest {
    fn is_subintent(&self) -> bool {
        match self {
            AnyManifest::V1(m) => m.is_subintent(),
            AnyManifest::SystemV1(m) => m.is_subintent(),
            AnyManifest::V2(m) => m.is_subintent(),
            AnyManifest::SubintentV2(m) => m.is_subintent(),
        }
    }

    fn get_blobs<'a>(&'a self) -> impl Iterator<Item = (&'a Hash, &'a Vec<u8>)> {
        let iterator: Box<dyn Iterator<Item = (&'a Hash, &'a Vec<u8>)> + 'a> = match self {
            AnyManifest::V1(m) => Box::new(m.get_blobs()),
            AnyManifest::SystemV1(m) => Box::new(m.get_blobs()),
            AnyManifest::V2(m) => Box::new(m.get_blobs()),
            AnyManifest::SubintentV2(m) => Box::new(m.get_blobs()),
        };
        iterator
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        match self {
            AnyManifest::V1(m) => m.get_known_object_names_ref(),
            AnyManifest::SystemV1(m) => m.get_known_object_names_ref(),
            AnyManifest::V2(m) => m.get_known_object_names_ref(),
            AnyManifest::SubintentV2(m) => m.get_known_object_names_ref(),
        }
    }

    fn get_preallocated_addresses(&self) -> &[PreAllocatedAddress] {
        match self {
            AnyManifest::V1(m) => m.get_preallocated_addresses(),
            AnyManifest::SystemV1(m) => m.get_preallocated_addresses(),
            AnyManifest::V2(m) => m.get_preallocated_addresses(),
            AnyManifest::SubintentV2(m) => m.get_preallocated_addresses(),
        }
    }

    fn get_child_subintent_hashes<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a ChildSubintentSpecifier> {
        let iterator: Box<dyn ExactSizeIterator<Item = &'a ChildSubintentSpecifier>> = match self {
            AnyManifest::V1(m) => Box::new(m.get_child_subintent_hashes()),
            AnyManifest::SystemV1(m) => Box::new(m.get_child_subintent_hashes()),
            AnyManifest::V2(m) => Box::new(m.get_child_subintent_hashes()),
            AnyManifest::SubintentV2(m) => Box::new(m.get_child_subintent_hashes()),
        };
        iterator
    }
}

impl ReadableManifest for AnyManifest {
    fn iter_instruction_effects(&self) -> impl Iterator<Item = ManifestInstructionEffect> {
        let iterator: Box<dyn Iterator<Item = ManifestInstructionEffect>> = match self {
            AnyManifest::V1(m) => Box::new(m.iter_instruction_effects()),
            AnyManifest::SystemV1(m) => Box::new(m.iter_instruction_effects()),
            AnyManifest::V2(m) => Box::new(m.iter_instruction_effects()),
            AnyManifest::SubintentV2(m) => Box::new(m.iter_instruction_effects()),
        };
        iterator
    }

    fn iter_cloned_instructions(&self) -> impl Iterator<Item = AnyInstruction> {
        let iterator: Box<dyn Iterator<Item = AnyInstruction>> = match self {
            AnyManifest::V1(m) => Box::new(m.iter_cloned_instructions()),
            AnyManifest::SystemV1(m) => Box::new(m.iter_cloned_instructions()),
            AnyManifest::V2(m) => Box::new(m.iter_cloned_instructions()),
            AnyManifest::SubintentV2(m) => Box::new(m.iter_cloned_instructions()),
        };
        iterator
    }

    fn instruction_count(&self) -> usize {
        match self {
            AnyManifest::V1(m) => m.instruction_count(),
            AnyManifest::SystemV1(m) => m.instruction_count(),
            AnyManifest::V2(m) => m.instruction_count(),
            AnyManifest::SubintentV2(m) => m.instruction_count(),
        }
    }

    fn instruction_effect(&self, index: usize) -> ManifestInstructionEffect {
        match self {
            AnyManifest::V1(m) => m.instruction_effect(index),
            AnyManifest::SystemV1(m) => m.instruction_effect(index),
            AnyManifest::V2(m) => m.instruction_effect(index),
            AnyManifest::SubintentV2(m) => m.instruction_effect(index),
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
