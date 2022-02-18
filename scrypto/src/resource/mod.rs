mod bucket;
mod non_fungible;
mod non_fungible_data;
mod non_fungible_key;
mod proof;
mod resource_builder;
mod resource_def;
mod resource_type;
mod supply;
mod vault;

/// Resource flags.
pub mod resource_flags;
/// Resource permissions.
pub mod resource_permissions;

pub use bucket::{Bucket, ParseBucketError};
pub use non_fungible::NonFungible;
pub use non_fungible_data::NonFungibleData;
pub use non_fungible_key::{NonFungibleKey, ParseNonFungibleKeyError};
pub use proof::{ParseProofError, Proof};
pub use resource_builder::{ResourceBuilder, DIVISIBILITY_MAXIMUM, DIVISIBILITY_NONE};
pub use resource_def::{ParseResourceDefIdError, ResourceDefId};
pub use resource_flags::*;
pub use resource_permissions::*;
pub use resource_type::ResourceType;
pub use supply::Supply;
pub use vault::{ParseVaultError, Vault};

use crate::engine::{api::*, call_engine};
use crate::rust::collections::HashMap;
use crate::rust::string::String;

/// Creates a resource with the given parameters.
///
/// A bucket is returned iif an initial supply is provided.
pub fn create_resource(
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    flags: u64,
    mutable_flags: u64,
    authorities: HashMap<ResourceDefId, u64>,
    initial_supply: Option<Supply>,
) -> (ResourceDefId, Option<Bucket>) {
    let input = CreateResourceInput {
        resource_type,
        metadata,
        flags,
        mutable_flags,
        authorities,
        initial_supply,
    };
    let output: CreateResourceOutput = call_engine(CREATE_RESOURCE, input);

    (
        output.resource_def_id,
        output.bucket_id.map(|id| Bucket(id)),
    )
}
