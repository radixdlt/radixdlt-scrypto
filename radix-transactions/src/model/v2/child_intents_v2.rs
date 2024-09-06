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
        let instruction = DecompiledInstruction::new("USE_CHILD")
            .add_argument(context.new_address_reservation())
            .add_raw_argument(format!("Intent(\"{subintent_id}\")"));
        Ok(instruction)
    }
}

/// A new-type representing the index of a referenced intent.
/// The first few of these will be the children of the given intent.
#[derive(Debug, Clone, Copy, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ManifestIntent(pub u32);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedChildIntentsV2 {
    pub children: Vec<ChildSubintent>,
    pub summary: Summary,
}

impl_has_summary!(PreparedChildIntentsV2);

impl TransactionPreparableFromValueBody for PreparedChildIntentsV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let (hashes, summary) = ConcatenatedDigest::prepare_from_sbor_array_value_body::<
            Vec<RawHash>,
            V2_MAX_NUMBER_OF_CHILD_SUBINTENTS_IN_INTENT,
        >(decoder, ValueType::ChildIntentConstraint)?;

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
