use crate::internal_prelude::*;

define_wrapped_hash!(
    /// Represents a particular schema under a package
    SchemaHash
);

/*
// NOTE: Conceptually we could have the following type, which can be used for _type identity_.
// This isn't currently needed in the engine however, so is commented out to avoid dead code.
```
/// A global identifier for a type in a Radix network.
/// A type is either well-known, or local to a node.
/// This identifier includes the NodeId, which provides context for how to look-up the type.
///
/// If/when we add additional type metadata (eg translations, documentation),
/// these will be added by the owner of the Node against the GlobalTypeAddress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum GlobalTypeAddress {
    WellKnown(WellKnownTypeId),
    NodeLocal(NodeId, SchemaHash, usize),
}
```
*/

/// An identifier for a type under a given node's schema context in the Radix network.
///
/// See also [`ScopedTypeId`] which captures an identifier for a type where the node
/// is clear from context.
///
/// Note - this type provides scoping to a schema even for well-known types where
/// the schema is irrelevant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct FullyScopedTypeId<T: AsRef<NodeId>>(pub T, pub SchemaHash, pub LocalTypeId);

impl<T: AsRef<NodeId>> FullyScopedTypeId<T> {
    pub fn into_general(self) -> FullyScopedTypeId<NodeId> {
        FullyScopedTypeId(*self.0.as_ref(), self.1, self.2)
    }
}

/// An identifier for a type in the context of a schema.
///
/// The location of the schema store is not given in this type, and
/// is known from context. Currently the schema store will be in the
/// Schema partition under a node.
///
/// See also [`FullyScopedTypeId`] for the same type, but with the node schema
/// location included.
///
/// Note - this type provides scoping to a schema even for well-known types where
/// the schema is irrelevant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct ScopedTypeId(pub SchemaHash, pub LocalTypeId);

impl ScopedTypeId {
    pub fn under_node<T: AsRef<NodeId>>(self, node: T) -> FullyScopedTypeId<T> {
        FullyScopedTypeId(node, self.0, self.1)
    }
}

/// A reference to the type to substitute with for the case of generics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, ManifestSbor, ScryptoSbor)]
pub enum GenericSubstitution {
    Local(ScopedTypeId),
    /// Currently supports default version of blueprint only.
    /// New variants can be added for specific version of blueprint in the future.
    Remote(BlueprintTypeIdentifier),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ManifestSbor, ScryptoSbor)]
pub struct BlueprintTypeIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub type_name: String,
}
