use crate::internal_prelude::*;

define_wrapped_hash!(
    /// Represents a particular schema under a package
    SchemaHash
);

/// A global identifier for a type in a Radix network.
/// A type is either well-known, or local to a node.
/// This identifier includes the NodeId, which provides context for how
/// to look-up the type.
///
/// If/when we add additional type metadata (eg translations, documentation),
/// these will be added by the owner of the Node against the GlobalTypeAddress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum GlobalTypeAddress {
    WellKnown(WellKnownTypeId),
    NodeLocal(NodeId, SchemaHash, usize),
}

/// An identifier for a type under a given schema context in the Radix network.
///
/// Note - this type provides scoping to a schema even for well-known types where
/// the schema is irrelevant.
///
/// See also:
/// * [`GlobalTypeAddress`] which captures the global type identity of a type,
/// for the purpose of semantics and ownership.
/// * [`NodeScopedTypeId`] which captures an identifier for a type where the node
/// is clear from context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct FullyScopedTypeId(pub NodeId, pub SchemaHash, pub LocalTypeId);

/// An identifier for a type in the context of a schema under a given node.
/// The given node is context-dependent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct NodeScopedTypeId(pub SchemaHash, pub LocalTypeId);

/// A reference to the type to substitute with for the case of generics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, ManifestSbor, ScryptoSbor)]
pub enum GenericSubstitution {
    Local(NodeScopedTypeId),
    Remote {
        package_address: PackageAddress,
        blueprint_name: String,
        type_name: String,
    },
}
