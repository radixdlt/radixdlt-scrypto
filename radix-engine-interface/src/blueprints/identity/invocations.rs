use crate::blueprints::component::*;
use crate::blueprints::resource::*;

use radix_common::prelude::*;
use sbor::rust::fmt::Debug;

pub const IDENTITY_BLUEPRINT: &str = "Identity";

define_type_marker!(Some(IDENTITY_PACKAGE), Identity);

pub const IDENTITY_CREATE_ADVANCED_IDENT: &str = "create_advanced";

#[cfg_attr(feature = "fuzzing", derive(::arbitrary::Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestEncode, ManifestCategorize)]
pub struct IdentityCreateAdvancedInput {
    pub owner_role: OwnerRole,
}

#[cfg_attr(feature = "fuzzing", derive(::arbitrary::Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct IdentityCreateAdvancedManifestInput {
    pub owner_role: ManifestOwnerRole,
}

pub type IdentityCreateAdvancedOutput = Global<IdentityMarker>;

pub const IDENTITY_CREATE_IDENT: &str = "create";

#[cfg_attr(feature = "fuzzing", derive(::arbitrary::Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct IdentityCreateInput {}

pub type IdentityCreateManifestInput = IdentityCreateInput;

pub type IdentityCreateOutput = (Global<IdentityMarker>, Bucket);

pub const IDENTITY_SECURIFY_IDENT: &str = "securify";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct IdentitySecurifyToSingleBadgeInput {}

pub type IdentitySecurifyToSingleBadgeManifestInput = IdentitySecurifyToSingleBadgeInput;

pub type IdentitySecurifyToSingleBadgeOutput = Bucket;
