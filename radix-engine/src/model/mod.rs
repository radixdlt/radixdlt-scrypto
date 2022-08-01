mod abi_extractor;
mod auth_converter;
mod auth_zone;
mod bucket;
mod component;
mod method_authorization;
mod non_fungible;
mod package_extractor;
mod proof;
mod resource;
mod resource_manager;
mod system;
mod transaction_processor;
mod validated_package;
mod vault;
mod worktop;

pub use abi_extractor::*;
pub use auth_converter::convert;
pub use auth_zone::{AuthZone, AuthZoneError};
pub use bucket::{Bucket, BucketError};
pub use component::{Component, ComponentError, ComponentState};
pub use method_authorization::{
    HardAuthRule, HardProofRule, HardResourceOrNonFungible, MethodAuthorization,
    MethodAuthorizationError,
};
pub use non_fungible::NonFungible;
pub use package_extractor::{extract_package, ExtractAbiError};
pub use proof::*;
pub use resource::*;
pub use resource_manager::{ResourceManager, ResourceManagerError};
pub use system::{System, SystemError};
pub use transaction_processor::{
    TransactionProcessor, TransactionProcessorError, TransactionProcessorRunInput,
};
pub use validated_package::{PackageError, ValidatedPackage};
pub use vault::{Vault, VaultError};
pub use worktop::{Worktop, WorktopError};

use sbor::rust::vec::Vec;
use sbor::*;

/// To support non-fungible deletion, we wrap it into a container
/// when persisting into the substate store.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NonFungibleWrapper(pub Option<NonFungible>);

/// To support key value store entry deletion, we wrap it into a container
/// when persisting into the substate store.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct KeyValueStoreEntryWrapper(pub Option<Vec<u8>>);
