use crate::prelude::*;

/// The creation strategy used in the [`BucketFactory`] and [`ProofFactory`] structs allowing them
/// to either create them by disabling auth and minting or by mocking the bucket or proof.
///
/// [`BucketFactory`]: crate::prelude::BucketFactory
/// [`ProofFactory`]: crate::prelude::ProofFactory
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreationStrategy {
    /// Disables the auth module and mints the specified amount of resources and then creates a
    /// [`Bucket`] or [`Proof`] out of them.
    ///
    /// [`Bucket`]: crate::prelude::Bucket
    /// [`Proof`]: crate::prelude::Proof
    DisableAuthAndMint,

    /// Mocks the creation of [`Bucket`]s and [`Proof`]s by creating nodes with their respective
    /// substates. This approach does not increase the total supply of the resource but can be
    /// somewhat fragile in some tests.
    ///
    /// [`Bucket`]: crate::prelude::Bucket
    /// [`Proof`]: crate::prelude::Proof
    Mock,
}
pub use CreationStrategy::*;

#[derive(Clone, Debug)]
pub enum FactoryResourceSpecifier {
    Amount(ResourceAddress, Decimal),
    Ids(ResourceAddress, IndexMap<NonFungibleLocalId, ScryptoValue>),
}

impl FactoryResourceSpecifier {
    pub fn resource_address(&self) -> &ResourceAddress {
        match self {
            Self::Amount(address, ..) | Self::Ids(address, ..) => address,
        }
    }
}
