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

pub trait ManifestPayload:
    Into<AnyManifest>
    + TryFrom<AnyManifest>
    + for<'a> ManifestSborEnumVariantFor<
        AnyManifest,
        OwnedVariant: ManifestDecode,
        BorrowedVariant<'a>: ManifestEncode,
    >
{
    fn to_raw(self) -> Result<RawManifest, EncodeError> {
        Ok(manifest_encode(&self.as_encodable_variant())?.into())
    }

    fn to_canonical_bytes(self) -> Result<Vec<u8>, EncodeError> {
        self.to_raw().map(|raw| raw.0)
    }

    fn from_raw(raw: &RawManifest) -> Result<Self, DecodeError> {
        Ok(Self::from_decoded_variant(manifest_decode(raw.as_ref())?))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        AnyManifest::attempt_decode_from_arbitrary_payload(bytes)?
            .try_into()
            .map_err(|_| {
                format!(
                    "Manifest wasn't of the expected type: {}.",
                    std::any::type_name::<Self>()
                )
            })
    }
}

impl<
        M: Into<AnyManifest>
            + TryFrom<AnyManifest>
            + for<'a> ManifestSborEnumVariantFor<
                AnyManifest,
                OwnedVariant: ManifestDecode,
                BorrowedVariant<'a>: ManifestEncode,
            >,
    > ManifestPayload for M
{
}

// It's not technically a conventional transaction payload, but let's reuse the macro
define_raw_transaction_payload!(RawManifest, TransactionPayloadKind::Other);

impl AnyManifest {
    pub fn to_raw(&self) -> Result<RawManifest, EncodeError> {
        Ok(RawManifest::from_vec(manifest_encode(self)?))
    }

    pub fn to_canonical_bytes(self) -> Result<Vec<u8>, EncodeError> {
        self.to_raw().map(|raw| raw.0)
    }

    pub fn from_raw(raw: &RawManifest) -> Result<Self, DecodeError> {
        Ok(manifest_decode(raw.as_slice())?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        AnyManifest::attempt_decode_from_arbitrary_payload(bytes)
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

#[cfg(test)]
mod tests {
    use crate::internal_prelude::*;

    #[test]
    pub fn subintent_manifest_v2_is_round_trip_encodable_and_fixed() {
        let builder = ManifestBuilder::new_subintent_v2();
        let lookup = builder.name_lookup();

        // We include an object name to check that gets preserved
        let manifest = builder
            .take_all_from_worktop(RORK, "my_bucket")
            .yield_to_parent((lookup.bucket("my_bucket"),))
            .build();
        let encoded = manifest.clone().to_raw().unwrap();
        let decoded = SubintentManifestV2::from_raw(&encoded).unwrap();
        assert_eq!(manifest, decoded);

        // Ensuring that old encoded manifests can be decoded is required to ensure that manifests
        // saved with `rtmc` can still be read with `rtmd`
        let cuttlefish_hex = "4d2203012104202202020180005da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c660012101810000000023202000202000220101230c230509616464726573736573090c00076275636b657473090c0100000000096d795f6275636b657407696e74656e7473090c000670726f6f6673090c000c7265736572766174696f6e73090c00";
        let cuttlefish_raw = RawManifest::from_hex(cuttlefish_hex).unwrap();
        let cuttlefish_decoded = SubintentManifestV2::from_raw(&cuttlefish_raw).unwrap();
        assert_eq!(manifest, cuttlefish_decoded);
    }

    #[test]
    pub fn transaction_intent_manifest_v2_is_round_trip_encodable_and_fixed() {
        let builder = ManifestBuilder::new_v2();

        // We include an object name to check that gets preserved
        let manifest = builder
            .lock_fee_from_faucet()
            .create_proof_from_auth_zone_of_all(RORK, "my_proof")
            .build();
        let encoded = manifest.clone().to_raw().unwrap();
        let decoded = TransactionManifestV2::from_raw(&encoded).unwrap();
        assert_eq!(manifest, decoded);

        // Ensuring that old encoded manifests can be decoded is required to ensure that manifests
        // saved with `rtmc` can still be read with `rtmd`
        let cuttlefish_hex = "4d220201210420220241038000c0566318c6318c64f798cacc6318c6318cf7be8af78a78f8a6318c6318c60c086c6f636b5f66656521018500002059dd64f00c0f010000000000000000000000000000160180005da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c623202000202000220101230c230509616464726573736573090c00076275636b657473090c0007696e74656e7473090c000670726f6f6673090c0100000000086d795f70726f6f660c7265736572766174696f6e73090c00";
        let cuttlefish_raw = RawManifest::from_hex(cuttlefish_hex).unwrap();
        let cuttlefish_decoded = TransactionManifestV2::from_raw(&cuttlefish_raw).unwrap();
        assert_eq!(manifest, cuttlefish_decoded);
    }
}
