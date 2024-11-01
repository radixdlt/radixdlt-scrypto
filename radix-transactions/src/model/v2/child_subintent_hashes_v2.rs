use crate::internal_prelude::*;
use decompiler::*;

/// Specification of an intent
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ChildSubintentSpecifiersV2 {
    pub children: IndexSet<ChildSubintentSpecifier>,
}

impl TransactionPartialPrepare for ChildSubintentSpecifiersV2 {
    type Prepared = PreparedChildSubintentSpecifiersV2V2;
}

/// A new-type of a [`SubintentHash`], representing that the subintent is claimed
/// to be a child of the given intent.
#[derive(Debug, Clone, Eq, Hash, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ChildSubintentSpecifier {
    pub hash: SubintentHash,
}

impl ChildSubintentSpecifier {
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

//========
// resolution
//========

/// This is for use with the [`ResolvableManifestNamedIntent`] trait.
/// Implementers should panic if a bucket cannot be found.
pub trait NamedManifestIntentResolver {
    fn assert_named_intent_exists(&self, named_intent: ManifestNamedIntent);
    fn resolve_named_intent(&self, name: &str) -> ManifestNamedIntent;
}

/// This trait is intended to be used as an `impl` argument in helper methods
/// operating on manifests, to resolve a [`ManifestNamedIntent`] from a name, an id,
/// or from itself.
///
/// The resolution process relies on a [`NamedManifestBucketResolver`] which can
/// provide a lookup to/from names.
pub trait ResolvableManifestNamedIntent {
    fn resolve(self, resolver: &impl NamedManifestIntentResolver) -> ManifestNamedIntent;
}

impl<A, E> ResolvableManifestNamedIntent for A
where
    A: TryInto<ManifestNamedIntent, Error = E>,
    E: Debug,
{
    fn resolve(self, resolver: &impl NamedManifestIntentResolver) -> ManifestNamedIntent {
        let named_intent = self
            .try_into()
            .expect("Value was not a valid ManifestNamedIntent");
        resolver.assert_named_intent_exists(named_intent);
        named_intent
    }
}

impl<'a> ResolvableManifestNamedIntent for &'a str {
    fn resolve(self, resolver: &impl NamedManifestIntentResolver) -> ManifestNamedIntent {
        resolver.resolve_named_intent(self).into()
    }
}

impl<'a> ResolvableManifestNamedIntent for &'a String {
    fn resolve(self, resolver: &impl NamedManifestIntentResolver) -> ManifestNamedIntent {
        resolver.resolve_named_intent(self.as_str()).into()
    }
}

impl ResolvableManifestNamedIntent for String {
    fn resolve(self, resolver: &impl NamedManifestIntentResolver) -> ManifestNamedIntent {
        resolver.resolve_named_intent(self.as_str()).into()
    }
}

//========
// prepared
//========

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedChildSubintentSpecifiersV2V2 {
    pub children: IndexSet<ChildSubintentSpecifier>,
    pub summary: Summary,
}

impl_has_summary!(PreparedChildSubintentSpecifiersV2V2);

impl TransactionPreparableFromValueBody for PreparedChildSubintentSpecifiersV2V2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let max_child_subintents_per_intent = decoder.settings().max_child_subintents_per_intent;
        let (hashes, summary) =
            ConcatenatedDigest::prepare_from_sbor_array_value_body::<Vec<RawHash>>(
                decoder,
                ValueType::ChildIntentConstraint,
                max_child_subintents_per_intent,
            )?;

        let mut children = index_set_with_capacity(hashes.len());
        for raw_hash in hashes {
            if !children.insert(ChildSubintentSpecifier {
                hash: SubintentHash::from_hash(raw_hash.hash),
            }) {
                return Err(PrepareError::DecodeError(DecodeError::DuplicateKey));
            }
        }

        Ok(Self { children, summary })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Array
    }
}
