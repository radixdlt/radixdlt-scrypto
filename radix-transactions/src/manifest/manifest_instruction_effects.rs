use crate::internal_prelude::*;

/// A nicer, grouped representation of a Transaction Instruction
#[derive(Debug, Clone, Copy)]
pub enum ManifestInstructionEffect<'a> {
    CreateBucket {
        source_amount: BucketSourceAmount<'a>,
    },
    CreateProof {
        source_amount: ProofSourceAmount<'a>,
    },
    ConsumeBucket {
        consumed_bucket: ManifestBucket,
        destination: BucketDestination<'a>,
    },
    ConsumeProof {
        consumed_proof: ManifestProof,
        destination: ProofDestination<'a>,
    },
    CloneProof {
        cloned_proof: ManifestProof,
    },
    DropManyProofs {
        drop_all_named_proofs: bool,
        drop_all_authzone_signature_proofs: bool,
        drop_all_authzone_non_signature_proofs: bool,
    },
    Invocation {
        kind: InvocationKind<'a>,
        args: &'a ManifestValue,
    },
    CreateAddressAndReservation {
        package_address: &'a PackageAddress,
        blueprint_name: &'a str,
    },
    ResourceAssertion {
        assertion: ResourceAssertion<'a>,
    },
    Verification {
        verification: VerificationKind,
        access_rule: &'a AccessRule,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum InvocationKind<'a> {
    Method {
        address: &'a ManifestGlobalAddress,
        module_id: ModuleId,
        method: &'a str,
    },
    Function {
        address: &'a ManifestPackageAddress,
        blueprint: &'a str,
        function: &'a str,
    },
    DirectMethod {
        address: &'a InternalAddress,
        method: &'a str,
    },
    YieldToParent,
    YieldToChild {
        child_index: ManifestNamedIntent,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum VerificationKind {
    Parent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BucketSourceAmount<'a> {
    AllOnWorktop {
        resource_address: &'a ResourceAddress,
    },
    AmountFromWorktop {
        resource_address: &'a ResourceAddress,
        amount: Decimal,
    },
    NonFungiblesFromWorktop {
        resource_address: &'a ResourceAddress,
        ids: &'a [NonFungibleLocalId],
    },
}

impl<'a> BucketSourceAmount<'a> {
    pub fn resource_address(&self) -> &'a ResourceAddress {
        match self {
            Self::AllOnWorktop { resource_address }
            | Self::AmountFromWorktop {
                resource_address, ..
            }
            | Self::NonFungiblesFromWorktop {
                resource_address, ..
            } => resource_address,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProofSourceAmount<'a> {
    AuthZonePopLastAddedProof,
    AuthZoneAllOf {
        resource_address: &'a ResourceAddress,
    },
    AuthZoneAmount {
        resource_address: &'a ResourceAddress,
        amount: Decimal,
    },
    AuthZoneNonFungibles {
        resource_address: &'a ResourceAddress,
        ids: &'a [NonFungibleLocalId],
    },
    BucketAllOf {
        bucket: ManifestBucket,
    },
    BucketAmount {
        bucket: ManifestBucket,
        amount: Decimal,
    },
    BucketNonFungibles {
        bucket: ManifestBucket,
        ids: &'a [NonFungibleLocalId],
    },
}

impl<'a> ProofSourceAmount<'a> {
    pub fn proof_kind(&self) -> ProofKind {
        match self {
            ProofSourceAmount::AuthZonePopLastAddedProof
            | ProofSourceAmount::AuthZoneAllOf { .. }
            | ProofSourceAmount::AuthZoneAmount { .. }
            | ProofSourceAmount::AuthZoneNonFungibles { .. } => ProofKind::AuthZoneProof,
            ProofSourceAmount::BucketAllOf { bucket, .. }
            | ProofSourceAmount::BucketAmount { bucket, .. }
            | ProofSourceAmount::BucketNonFungibles { bucket, .. } => {
                ProofKind::BucketProof(*bucket)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BucketDestination<'a> {
    Worktop,
    Burned,
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum ProofDestination<'a> {
    AuthZone,
    Drop,
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum AddressReservationDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum ExpressionDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum BlobDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceAssertion<'a> {
    Worktop(WorktopAssertion<'a>),
    NextCall(NextCallAssertion<'a>),
    Bucket(BucketAssertion<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum WorktopAssertion<'a> {
    ResourceNonZeroAmount {
        resource_address: &'a ResourceAddress,
    },
    ResourceAtLeastAmount {
        resource_address: &'a ResourceAddress,
        amount: Decimal,
    },
    ResourceAtLeastNonFungibles {
        resource_address: &'a ResourceAddress,
        ids: &'a [NonFungibleLocalId],
    },
    ResourcesOnly {
        constraints: &'a ManifestResourceConstraints,
    },
    ResourcesInclude {
        constraints: &'a ManifestResourceConstraints,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum NextCallAssertion<'a> {
    ReturnsOnly {
        constraints: &'a ManifestResourceConstraints,
    },
    ReturnsInclude {
        constraints: &'a ManifestResourceConstraints,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BucketAssertion<'a> {
    Contents {
        bucket: ManifestBucket,
        constraint: &'a ManifestResourceConstraint,
    },
}

pub trait HasManifestModel: ScryptoEncode + ScryptoDecode {
    type ManifestModel: ManifestModel;
}

pub trait ManifestModel: ManifestEncode + ManifestDecode + ManifestFlexibleCategorize {}
impl<T: ManifestEncode + ManifestDecode + ManifestFlexibleCategorize> ManifestModel for T {}

pub trait ManifestFlexibleCategorize: Sized {
    /// The type of the flexible array holding this type as an item
    type FlexibleItemedArray<A: ManifestArrayChoice>: ManifestFlexibleCategorize;
    /// The type of the flexible outer-half of the map holding this type as a key
    type FlexibleOuterKeyedMap<MC: ManifestMapChoice, V: ManifestFlexibleCategorize>: FlexibleOuterKeyedMapFromInnerValuedMap<MC, Self, V>;
    /// The type of the flexible inner-half of the map holding this type as a value
    type FlexibleInnerValuedMap<MC: ManifestMapChoice, K: ManifestFlexibleCategorize>: FlexibleInnerValuedMapFromMap<MC, K, Self>;
}
macro_rules! standard_flexible_categorize {
    ($t:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? ManifestFlexibleCategorize for $t $(< $( $lt ),+ >)? {
            // Use weird names to avoid clashes
            type FlexibleItemedArray<__AC: ManifestArrayChoice> = __AC::Array<Self>;
            type FlexibleOuterKeyedMap<__MC: ManifestMapChoice, __V: ManifestFlexibleCategorize> = __V::FlexibleInnerValuedMap<__MC, Self>;
            type FlexibleInnerValuedMap<__MC: ManifestMapChoice, __K: ManifestFlexibleCategorize> = __MC::Map<__K, Self>;
        }
    };
}

// Array Choice

pub trait ManifestArrayChoice: core::hash::Hash + PartialEq + Eq {
    type Array<T: ManifestFlexibleCategorize>: ManifestFlexibleCategorize;
}

#[derive(Hash, PartialEq, Eq)]
pub struct VecChoice;
impl ManifestArrayChoice for VecChoice {
    type Array<T: ManifestFlexibleCategorize> = Vec<T>;
}
standard_flexible_categorize!(Vec<T: ManifestFlexibleCategorize>);

#[derive(Hash, PartialEq, Eq)]
pub struct ArrayChoice<const N: usize>;
impl<const N: usize> ManifestArrayChoice for ArrayChoice<N> {
    type Array<T: ManifestFlexibleCategorize> = [T; N];
}

// Can't get macro working with const generics
impl<const N: usize, T: ManifestFlexibleCategorize> ManifestFlexibleCategorize for [T; N] {
    type FlexibleItemedArray<A: ManifestArrayChoice> = A::Array<Self>;
    type FlexibleOuterKeyedMap<M: ManifestMapChoice, V: ManifestFlexibleCategorize> =
        V::FlexibleInnerValuedMap<M, Self>;
    type FlexibleInnerValuedMap<MC: ManifestMapChoice, K: ManifestFlexibleCategorize> =
        MC::Map<K, Self>;
}

// Map Choice

pub trait ManifestMapChoice: core::hash::Hash + PartialEq + Eq {
    type Map<K: ManifestFlexibleCategorize, V: ManifestFlexibleCategorize>: ManifestMap<
        Choice = Self,
        Key = K,
        Value = V,
    >;
}

/// Represents exactly the map M<K, V>
pub trait ManifestMap:
    ManifestFlexibleCategorize + FlexibleInnerValuedMapFromMap<Self::Choice, Self::Key, Self::Value>
{
    type Choice: ManifestMapChoice<Map<Self::Key, Self::Value> = Self>;
    type Key: ManifestFlexibleCategorize;
    type Value: ManifestFlexibleCategorize;
}

pub trait FlexibleOuterKeyedMapFromInnerValuedMap<
    MC: ManifestMapChoice,
    K: ManifestFlexibleCategorize,
    V: ManifestFlexibleCategorize,
>: ManifestFlexibleCategorize
{
    fn construct_outer(inner_map: V::FlexibleInnerValuedMap<MC, K>) -> Self;
}

// We can't do a blanket implementation for V::FlexibleInnerValuedMap<MC, K> here because it
// conflicts with other trait definitions, even though they're guaranteed not to actually conflict >:(
// Instead, we need to do it for concrete types..

impl<
        MC: ManifestMapChoice,
        K: ManifestFlexibleCategorize,
        V: ManifestFlexibleCategorize<FlexibleInnerValuedMap<MC, K> = T>,
        T,
    > FlexibleOuterKeyedMapFromInnerValuedMap<MC, K, V> for T
{
    fn construct_outer(inner_map: V::FlexibleInnerValuedMap<MC, K>) -> Self {
        inner_map
    }
}

pub trait FlexibleInnerValuedMapFromMap<
    MC: ManifestMapChoice,
    K: ManifestFlexibleCategorize,
    V: ManifestFlexibleCategorize,
>: ManifestFlexibleCategorize
{
    fn construct_inner(map: MC::Map<K, V>) -> Self;
}

impl<M: ManifestMap> FlexibleInnerValuedMapFromMap<M::Choice, M::Key, M::Value> for M {
    fn construct_inner(map: <M::Choice as ManifestMapChoice>::Map<M::Key, M::Value>) -> Self {
        map
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct IndexMapChoice;
impl ManifestMapChoice for IndexMapChoice {
    type Map<K: ManifestFlexibleCategorize, V: ManifestFlexibleCategorize> = IndexMap<K, V>;
}
standard_flexible_categorize!(IndexMap<K: ManifestFlexibleCategorize, V: ManifestFlexibleCategorize>);
impl<K: ManifestFlexibleCategorize, V: ManifestFlexibleCategorize> ManifestMap for IndexMap<K, V> {
    type Key = K;
    type Value = V;
    type Choice = IndexMapChoice;
}

/// EXAMPLES: ManifestProof

impl HasManifestModel for Proof {
    type ManifestModel = ManifestProof;
}

impl ManifestFlexibleCategorize for ManifestProof {
    type FlexibleItemedArray<A: ManifestArrayChoice> = ManifestProofBatch<A>;
    type FlexibleOuterKeyedMap<M: ManifestMapChoice, V: ManifestFlexibleCategorize> =
        V::FlexibleInnerValuedMap<M, Self>;
    type FlexibleInnerValuedMap<MC: ManifestMapChoice, K: ManifestFlexibleCategorize> =
        MC::Map<K, Self>;
}

#[derive(ManifestSbor, Hash, PartialEq, Eq)]
#[sbor(child_types = "A::Array<ManifestProof>")]
pub enum ManifestProofBatch<A: ManifestArrayChoice = VecChoice> {
    ManifestProofs(A::Array<ManifestProof>),
    EntireAuthZone,
}

impl<AInner: ManifestArrayChoice> ManifestFlexibleCategorize for ManifestProofBatch<AInner> {
    type FlexibleItemedArray<AOuter: ManifestArrayChoice> = ManifestProofBatchArray<AOuter>;
    type FlexibleOuterKeyedMap<M: ManifestMapChoice, V: ManifestFlexibleCategorize> =
        FlexibleOuterManifestProofBatchKeyedMap<M, V, AInner>;
    type FlexibleInnerValuedMap<M: ManifestMapChoice, K: ManifestFlexibleCategorize> =
        FlexibleInnerManifestProofBatchValuedMap<M, K, AInner>;
}

#[derive(ManifestSbor, Hash, PartialEq, Eq)]
#[sbor(
    as_type = "ManifestExpression",
    as_ref = "{ #[allow(path_statements)]{ self; } &ManifestExpression::EntireAuthZone }",
    from_value = "{ #[allow(path_statements)]{ value; }; Self }"
)]
pub struct ManifestEntireAuthZoneExpression;
standard_flexible_categorize!(ManifestEntireAuthZoneExpression);

#[derive(ManifestSbor)]
#[sbor(
    child_types = "AOuter::Array<ManifestProofBatch<AInner>>; AOuter::Array<ManifestEntireAuthZoneExpression>"
)]
pub enum ManifestProofBatchArray<
    AOuter: ManifestArrayChoice = VecChoice,
    AInner: ManifestArrayChoice = VecChoice,
> {
    ManifestProofs(AOuter::Array<ManifestProofBatch<AInner>>),
    Expressions(AOuter::Array<ManifestEntireAuthZoneExpression>),
}
standard_flexible_categorize!(ManifestProofBatchArray<AOuter: ManifestArrayChoice, AInner: ManifestArrayChoice>);

#[derive(ManifestSbor)]
#[sbor(
    child_types = "V::FlexibleInnerValuedMap<MC, ManifestProofBatch<KeyAC>>; V::FlexibleInnerValuedMap<MC, ManifestEntireAuthZoneExpression>"
)]
pub enum FlexibleOuterManifestProofBatchKeyedMap<
    MC: ManifestMapChoice,
    V: ManifestFlexibleCategorize,
    KeyAC: ManifestArrayChoice = VecChoice,
> {
    ManifestProofs(V::FlexibleInnerValuedMap<MC, ManifestProofBatch<KeyAC>>),
    Expressions(V::FlexibleInnerValuedMap<MC, ManifestEntireAuthZoneExpression>),
}
standard_flexible_categorize!(FlexibleOuterManifestProofBatchKeyedMap<M: ManifestMapChoice, V: ManifestFlexibleCategorize, AInner: ManifestArrayChoice>);

// This will clash with the duplicate below; AND also with the blanket implementation above
// We can separate them out by using a helper trait as per https://github.com/rust-lang/rfcs/pull/1672#issuecomment-1405377983
// by requiring that flexible-inner-maps are distinct from flexible-outer-map-overrides; possibly via an "is outer map" override
impl<MC: ManifestMapChoice, KeyAC: ManifestArrayChoice, V: ManifestFlexibleCategorize>
    FlexibleOuterKeyedMapFromInnerValuedMap<MC, ManifestProofBatch<KeyAC>, V>
    for FlexibleOuterManifestProofBatchKeyedMap<MC, V, KeyAC>
{
    fn construct_outer(
        inner_map: V::FlexibleInnerValuedMap<MC, ManifestProofBatch<KeyAC>>,
    ) -> Self {
        FlexibleOuterManifestProofBatchKeyedMap::ManifestProofs(inner_map)
    }
}

// This doesn't work _at all_ because:
// > type parameter `T` must be used as the type parameter for some local type (e.g., `MyStruct<T>`)
// > implementing a foreign trait is only possible if at least one of the types for which it is implemented is local
// > only traits defined in the current crate can be implemented for a type
//
// Using Into we get the same issue.
//
// Implementing From onto the concrete types gives its own issue:
//     impl<
//         M: ManifestMap<Key = ManifestProofBatch<AInner>>,
//         AInner: ManifestArrayChoice,
//      > From<M> for FlexibleOuterManifestProofBatchKeyedMap<M::Choice, M::Value, AInner>
//
// > conflicting implementations of trait `std::convert::From<_>` for type `FlexibleOuterManifestProofBatchKeyedMap<_, _, _>`
// > conflicting implementation for `FlexibleOuterManifestProofBatchKeyedMap<_, _, _>`
//
// I think the only solution left to explore using our own, local, `FlexibleFrom` trait.
//
// BUT we should test it with also ensuring we can implement a general-case From<M> for T
//
// ...A FlexibleFrom might need partitioning by the kind of the resultant type:
// * (Outer)Map (which covers a blanket impl over Self and ManifestMap<Key = K>)
// * Array (which covers a blanket impl over Self and ManifestArray<Item = I>)
// * Derived struct/enum (which covers???)
impl<M: ManifestMap, T: FlexibleFromMapByKeyDiscriminator<M::Key, M>> From<M> for T {
    fn from(value: M) -> Self {
        T::flexible_from_map(value)
    }
}

// See helper here https://github.com/rust-lang/rfcs/pull/1672#issuecomment-1405377983
// * We can blanket implement this for distinct concrete K, as they are considered separate traits.
// * We can then implement some other trait by blanket-implementing it and using some partitioning
//   by the helper trait key.
trait FlexibleFromMapByKeyDiscriminator<K, M: ManifestMap<Key = K>> {
    fn flexible_from_map(map: M) -> Self;
}

impl<M: ManifestMap<Key = ManifestProofBatch<AInner>>, AInner: ManifestArrayChoice>
    FlexibleFromMapByKeyDiscriminator<ManifestProofBatch<AInner>, M>
    for FlexibleOuterManifestProofBatchKeyedMap<M::Choice, M::Value, AInner>
{
    fn flexible_from_map(map: M) -> Self {
        let inner = <<M::Value as ManifestFlexibleCategorize>::FlexibleInnerValuedMap<
            M::Choice,
            M::Key,
        > as FlexibleInnerValuedMapFromMap<M::Choice, M::Key, M::Value>>::construct_inner(
            map
        );
        FlexibleOuterManifestProofBatchKeyedMap::ManifestProofs(inner)
    }
}

impl<M: ManifestMap<Key = ManifestProofBatch<AInner>>, AInner: ManifestArrayChoice> From<M>
    for FlexibleOuterManifestProofBatchKeyedMap<M::Choice, M::Value, AInner>
{
    fn from(map: M) -> Self {
        let inner = <<M::Value as ManifestFlexibleCategorize>::FlexibleInnerValuedMap<
            M::Choice,
            M::Key,
        > as FlexibleInnerValuedMapFromMap<M::Choice, M::Key, M::Value>>::construct_inner(
            map
        );
        FlexibleOuterManifestProofBatchKeyedMap::ManifestProofs(inner)
    }
}

impl<M: ManifestMap<Key = ManifestEntireAuthZoneExpression>, AInner: ManifestArrayChoice> From<M>
    for FlexibleOuterManifestProofBatchKeyedMap<M::Choice, M::Value, AInner>
{
    fn from(map: M) -> Self {
        let inner = <<M::Value as ManifestFlexibleCategorize>::FlexibleInnerValuedMap<
            M::Choice,
            M::Key,
        > as FlexibleInnerValuedMapFromMap<M::Choice, M::Key, M::Value>>::construct_inner(
            map
        );
        FlexibleOuterManifestProofBatchKeyedMap::Expressions(inner)
    }
}

impl<M: ManifestMap<Key = ManifestEntireAuthZoneExpression>, AInner: ManifestArrayChoice>
    FlexibleFromMapByKeyDiscriminator<ManifestEntireAuthZoneExpression, M>
    for FlexibleOuterManifestProofBatchKeyedMap<M::Choice, M::Value, AInner>
{
    fn flexible_from_map(map: M) -> Self {
        let inner = <<M::Value as ManifestFlexibleCategorize>::FlexibleInnerValuedMap<
            M::Choice,
            M::Key,
        > as FlexibleInnerValuedMapFromMap<M::Choice, M::Key, M::Value>>::construct_inner(
            map
        );
        FlexibleOuterManifestProofBatchKeyedMap::Expressions(inner)
    }
}

#[derive(ManifestSbor)]
#[sbor(
    child_types = "M::Map<K, ManifestProofBatch<AInner>>; M::Map<K, Vec<ManifestEntireAuthZoneExpression>>"
)]
pub enum FlexibleInnerManifestProofBatchValuedMap<
    M: ManifestMapChoice,
    K: ManifestFlexibleCategorize,
    AInner: ManifestArrayChoice = VecChoice,
> {
    ManifestProofs(M::Map<K, ManifestProofBatch<AInner>>),
    Expressions(M::Map<K, Vec<ManifestEntireAuthZoneExpression>>),
}
standard_flexible_categorize!(FlexibleInnerManifestProofBatchValuedMap<M: ManifestMapChoice, K: ManifestFlexibleCategorize, AInner: ManifestArrayChoice>);

impl<MC: ManifestMapChoice, K: ManifestFlexibleCategorize, AInner: ManifestArrayChoice>
    FlexibleInnerValuedMapFromMap<MC, K, ManifestProofBatch<AInner>>
    for FlexibleInnerManifestProofBatchValuedMap<MC, K, AInner>
{
    fn construct_inner(map: MC::Map<K, ManifestProofBatch<AInner>>) -> Self {
        FlexibleInnerManifestProofBatchValuedMap::ManifestProofs(map)
    }
}

impl<MC: ManifestMapChoice, K: ManifestFlexibleCategorize, AInner: ManifestArrayChoice>
    FlexibleInnerValuedMapFromMap<MC, K, Vec<ManifestEntireAuthZoneExpression>>
    for FlexibleInnerManifestProofBatchValuedMap<MC, K, AInner>
{
    fn construct_inner(map: MC::Map<K, Vec<ManifestEntireAuthZoneExpression>>) -> Self {
        FlexibleInnerManifestProofBatchValuedMap::Expressions(map)
    }
}

impl<T: HasManifestModel + ScryptoCategorize> HasManifestModel for Vec<T>
where
    <T::ManifestModel as ManifestFlexibleCategorize>::FlexibleItemedArray<VecChoice>: ManifestModel,
{
    type ManifestModel =
        <T::ManifestModel as ManifestFlexibleCategorize>::FlexibleItemedArray<VecChoice>;
}

impl<
        K: HasManifestModel + ScryptoCategorize + core::hash::Hash + core::cmp::Eq,
        V: HasManifestModel + ScryptoCategorize,
    > HasManifestModel for IndexMap<K, V>
where
    <K::ManifestModel as ManifestFlexibleCategorize>::FlexibleOuterKeyedMap<
        IndexMapChoice,
        V::ManifestModel,
    >: ManifestModel,
{
    type ManifestModel = <K::ManifestModel as ManifestFlexibleCategorize>::FlexibleOuterKeyedMap<
        IndexMapChoice,
        V::ManifestModel,
    >;
}

type ToManifest<T> = <T as HasManifestModel>::ManifestModel;

#[cfg(test)]
mod tests {
    use super::*;

    fn conversion_check<T: HasManifestModel<ManifestModel = X>, X>() {}

    #[test]
    fn tests() {
        conversion_check::<Vec<Proof>, ManifestProofBatch<VecChoice>>();
        conversion_check::<Vec<Vec<Proof>>, ManifestProofBatchArray<VecChoice, VecChoice>>();
        conversion_check::<
            IndexMap<Vec<Proof>, Vec<Proof>>,
            FlexibleOuterManifestProofBatchKeyedMap<
                IndexMapChoice,
                ManifestProofBatch<VecChoice>,
                VecChoice,
            >,
        >();

        let value: ToManifest<IndexMap<Vec<Proof>, Vec<Proof>>> =
            FlexibleOuterManifestProofBatchKeyedMap::Expressions(
                FlexibleInnerManifestProofBatchValuedMap::ManifestProofs(IndexMap::<
                    ManifestEntireAuthZoneExpression,
                    ManifestProofBatch,
                >::new()),
            )
            .into();
        let value: ToManifest<IndexMap<Vec<Proof>, Vec<Proof>>> =
            IndexMap::<ManifestEntireAuthZoneExpression, ManifestProofBatch>::new().into();
        let value: ToManifest<IndexMap<Vec<Proof>, Proof>> =
            IndexMap::<ManifestEntireAuthZoneExpression, ManifestProof>::new().into();
    }
}
