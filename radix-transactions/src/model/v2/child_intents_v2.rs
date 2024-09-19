use crate::internal_prelude::*;
use decompiler::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ChildIntentsV2 {
    pub children: Vec<ChildSubintent>,
}

impl TransactionPartialPrepare for ChildIntentsV2 {
    type Prepared = PreparedChildIntentsV2;
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ChildSubintent {
    pub hash: SubintentHash,
}

impl ChildSubintent {
    pub fn decompile_as_pseudo_instruction(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let subintent_id = self.hash.to_string(context.transaction_hash_encoder());
        let new_named_intent = context.new_named_intent();
        let intent_name = context.object_names.intent_name(new_named_intent);
        let instruction = DecompiledInstruction::new("USE_CHILD")
            .add_raw_argument(format!("NamedIntent(\"{intent_name}\")"))
            .add_raw_argument(format!("Intent(\"{subintent_id}\")"));
        Ok(instruction)
    }
}

/// A new-type representing the index of a referenced intent.
/// The first few of these will be the children of the given intent.
///
/// This is referenced in the manifest as `NamedIntent`, like `NamedAddress`.
/// A static intent address is created as e.g. `Intent("subtxid_...")`, like `Address`.
///
/// IMPORTANT: Unlike `Address` and similar, this is NOT its own SBOR manifest value
/// - because versioning Manifest SBOR was seen as too much work for Cuttlefish.
/// Instead, we use a ManifestNamedIntentIndex in some places.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ManifestNamedIntent(pub u32);

/// This exists as an unideal serialization target for [`ManifestNamedIntent`],
/// due to our inability to add a new ManifestCustomValue for the Cuttlefish update.
/// Instead, we just serialize it directly as a `u32` in the `YIELD_TO_CHILD` instruction.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, ScryptoDescribe, ManifestSbor)]
#[sbor(transparent)]
pub struct ManifestNamedIntentIndex(pub u32);

impl From<ManifestNamedIntentIndex> for ManifestNamedIntent {
    fn from(value: ManifestNamedIntentIndex) -> Self {
        Self(value.0)
    }
}

impl From<ManifestNamedIntent> for ManifestNamedIntentIndex {
    fn from(value: ManifestNamedIntent) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedChildIntentsV2 {
    pub children: Vec<ChildSubintent>,
    pub summary: Summary,
}

impl_has_summary!(PreparedChildIntentsV2);

impl TransactionPreparableFromValueBody for PreparedChildIntentsV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let max_child_subintents_per_intent = decoder.settings().max_child_subintents_per_intent;
        let (hashes, summary) =
            ConcatenatedDigest::prepare_from_sbor_array_value_body::<Vec<RawHash>>(
                decoder,
                ValueType::ChildIntentConstraint,
                max_child_subintents_per_intent,
            )?;

        Ok(Self {
            children: hashes
                .into_iter()
                .map(|h| ChildSubintent {
                    hash: SubintentHash::from_hash(h.hash),
                })
                .collect(),
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Array
    }
}
