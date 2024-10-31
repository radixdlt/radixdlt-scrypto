use crate::blueprints::resource::CompositeRequirement::{AllOf, AnyOf};
use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum ResourceOrNonFungible {
    NonFungible(NonFungibleGlobalId),
    Resource(ResourceAddress),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ManifestSbor)]
pub enum ManifestResourceOrNonFungible {
    NonFungible(NonFungibleGlobalId),
    Resource(ManifestResourceAddress),
}

impl From<ResourceOrNonFungible> for ManifestResourceOrNonFungible {
    fn from(value: ResourceOrNonFungible) -> Self {
        match value {
            ResourceOrNonFungible::NonFungible(non_fungible_global_id) => {
                Self::NonFungible(non_fungible_global_id)
            }
            ResourceOrNonFungible::Resource(resource_address) => {
                Self::Resource(ManifestResourceAddress::Static(resource_address))
            }
        }
    }
}

impl Describe<ScryptoCustomTypeKind> for ResourceOrNonFungible {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::RESOURCE_OR_NON_FUNGIBLE_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::resource_or_non_fungible_type_data()
    }
}

impl From<NonFungibleGlobalId> for ResourceOrNonFungible {
    fn from(non_fungible_global_id: NonFungibleGlobalId) -> Self {
        ResourceOrNonFungible::NonFungible(non_fungible_global_id)
    }
}

impl From<ResourceAddress> for ResourceOrNonFungible {
    fn from(resource_address: ResourceAddress) -> Self {
        ResourceOrNonFungible::Resource(resource_address)
    }
}

pub struct ResourceOrNonFungibleList {
    list: Vec<ResourceOrNonFungible>,
}

impl<T> From<Vec<T>> for ResourceOrNonFungibleList
where
    T: Into<ResourceOrNonFungible>,
{
    fn from(addresses: Vec<T>) -> Self {
        ResourceOrNonFungibleList {
            list: addresses.into_iter().map(|a| a.into()).collect(),
        }
    }
}

/// Resource Proof Rules
#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum BasicRequirement {
    Require(ResourceOrNonFungible),
    AmountOf(Decimal, ResourceAddress),
    CountOf(u8, Vec<ResourceOrNonFungible>),
    AllOf(Vec<ResourceOrNonFungible>),
    AnyOf(Vec<ResourceOrNonFungible>),
}

impl Describe<ScryptoCustomTypeKind> for BasicRequirement {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::BASIC_REQUIREMENT_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::basic_requirement_type_data()
    }
}

impl From<ResourceAddress> for CompositeRequirement {
    fn from(resource_address: ResourceAddress) -> Self {
        CompositeRequirement::BasicRequirement(BasicRequirement::Require(resource_address.into()))
    }
}

impl From<NonFungibleGlobalId> for CompositeRequirement {
    fn from(id: NonFungibleGlobalId) -> Self {
        CompositeRequirement::BasicRequirement(BasicRequirement::Require(id.into()))
    }
}

impl From<ResourceOrNonFungible> for CompositeRequirement {
    fn from(resource_or_non_fungible: ResourceOrNonFungible) -> Self {
        CompositeRequirement::BasicRequirement(BasicRequirement::Require(resource_or_non_fungible))
    }
}

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum CompositeRequirement {
    BasicRequirement(BasicRequirement),
    AnyOf(Vec<CompositeRequirement>),
    AllOf(Vec<CompositeRequirement>),
}

impl Describe<ScryptoCustomTypeKind> for CompositeRequirement {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::COMPOSITE_REQUIREMENT_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::composite_requirement_type_data()
    }
}

impl CompositeRequirement {
    pub fn or(self, other: CompositeRequirement) -> Self {
        match self {
            CompositeRequirement::AnyOf(mut rules) => {
                rules.push(other);
                AnyOf(rules)
            }
            _ => AnyOf(vec![self, other]),
        }
    }

    pub fn and(self, other: CompositeRequirement) -> Self {
        match self {
            CompositeRequirement::AllOf(mut rules) => {
                rules.push(other);
                AllOf(rules)
            }
            _ => AllOf(vec![self, other]),
        }
    }
}

/// A requirement for the immediate caller's package to equal the given package.
pub fn package_of_direct_caller(package: PackageAddress) -> ResourceOrNonFungible {
    ResourceOrNonFungible::NonFungible(NonFungibleGlobalId::package_of_direct_caller_badge(package))
}

/// A requirement for the global ancestor of the actor who made the latest global call to either be:
/// * The main module of the given global component (pass a `ComponentAddress` or `GlobalAddress`)
/// * A package function on the given blueprint (pass `(PackageAddress, String)` or `Blueprint`)
pub fn global_caller(global_caller: impl Into<GlobalCaller>) -> ResourceOrNonFungible {
    ResourceOrNonFungible::NonFungible(NonFungibleGlobalId::global_caller_badge(global_caller))
}

/// A requirement for the transaction to be signed using a specific key.
pub fn signature(public_key: &impl HasPublicKeyHash) -> ResourceOrNonFungible {
    ResourceOrNonFungible::NonFungible(NonFungibleGlobalId::from_public_key(public_key))
}

/// A requirement for the transaction to be a system transaction.
pub fn system_execution(transaction_type: SystemExecution) -> NonFungibleGlobalId {
    transaction_type.into()
}

pub fn require<T>(required: T) -> CompositeRequirement
where
    T: Into<CompositeRequirement>,
{
    required.into()
}

pub fn require_any_of<T>(resources: T) -> CompositeRequirement
where
    T: Into<ResourceOrNonFungibleList>,
{
    let list: ResourceOrNonFungibleList = resources.into();
    CompositeRequirement::BasicRequirement(BasicRequirement::AnyOf(list.list))
}

pub fn require_all_of<T>(resources: T) -> CompositeRequirement
where
    T: Into<ResourceOrNonFungibleList>,
{
    let list: ResourceOrNonFungibleList = resources.into();
    CompositeRequirement::BasicRequirement(BasicRequirement::AllOf(list.list))
}

pub fn require_n_of<C, T>(count: C, resources: T) -> CompositeRequirement
where
    C: Into<u8>,
    T: Into<ResourceOrNonFungibleList>,
{
    let list: ResourceOrNonFungibleList = resources.into();
    CompositeRequirement::BasicRequirement(BasicRequirement::CountOf(count.into(), list.list))
}

pub fn require_amount<D, T>(amount: D, resource: T) -> CompositeRequirement
where
    D: Into<Decimal>,
    T: Into<ResourceAddress>,
{
    CompositeRequirement::BasicRequirement(BasicRequirement::AmountOf(
        amount.into(),
        resource.into(),
    ))
}

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum AccessRule {
    AllowAll,
    DenyAll,
    Protected(CompositeRequirement),
}

impl Describe<ScryptoCustomTypeKind> for AccessRule {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::ACCESS_RULE_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::access_rule_type_data()
    }
}

impl From<CompositeRequirement> for AccessRule {
    fn from(value: CompositeRequirement) -> Self {
        AccessRule::Protected(value)
    }
}

pub trait AccessRuleVisitor {
    type Error;
    fn visit(&mut self, node: &CompositeRequirement, depth: usize) -> Result<(), Self::Error>;
}

impl AccessRule {
    pub fn dfs_traverse_nodes<V: AccessRuleVisitor>(
        &self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        match self {
            AccessRule::Protected(node) => node.dfs_traverse_recursive(visitor, 0),
            _ => Ok(()),
        }
    }
}

impl CompositeRequirement {
    fn dfs_traverse_recursive<V: AccessRuleVisitor>(
        &self,
        visitor: &mut V,
        depth: usize,
    ) -> Result<(), V::Error> {
        visitor.visit(self, depth)?;

        match self {
            CompositeRequirement::BasicRequirement(..) => {}
            CompositeRequirement::AnyOf(nodes) | CompositeRequirement::AllOf(nodes) => {
                for node in nodes {
                    node.dfs_traverse_recursive(visitor, depth + 1)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_common::prelude::*;

    #[test]
    fn require_signature_secp256k1() {
        let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let public_key = private_key.public_key();

        let r1 = rule!(require(NonFungibleGlobalId::from_public_key(&public_key)));
        let r2 = rule!(require(signature(&public_key)));

        assert_eq!(r1, r2);
    }

    #[test]
    fn require_signature_ed25519() {
        let private_key = Ed25519PrivateKey::from_u64(1).unwrap();
        let public_key = private_key.public_key();

        let r1 = rule!(require(NonFungibleGlobalId::from_public_key(&public_key)));
        let r2 = rule!(require(signature(&public_key)));

        assert_eq!(r1, r2);
    }
}
