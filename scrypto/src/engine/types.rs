// Ideally, only the types listed below can be used by Radix Engine.
// We need a better strategy to enforce this.

pub use crate::component::ComponentId;
pub use crate::component::PackageId;
pub use crate::core::Actor;
pub use crate::core::Level;
pub use crate::crypto::EcdsaPublicKey;
pub use crate::crypto::Hash;
pub use crate::math::BigDecimal;
pub use crate::math::Decimal;
pub use crate::resource::NonFungibleId;
pub use crate::resource::ResourceDefId;
pub use crate::resource::ResourceType;
pub use crate::resource::Supply;

pub type LazyMapId = (Hash, u32);
pub type BucketId = u32;
pub type ProofId = u32;
pub type VaultId = (Hash, u32);

pub use crate::constants::*;
