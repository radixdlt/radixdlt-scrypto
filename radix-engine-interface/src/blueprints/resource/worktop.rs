use crate::blueprints::resource::*;
use crate::internal_prelude::*;
use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::*;
use radix_common::math::Decimal;
use radix_common::types::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const WORKTOP_BLUEPRINT: &str = "Worktop";

pub const WORKTOP_DROP_IDENT: &str = "Worktop_drop";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopDropInput {
    pub worktop: OwnedWorktop,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct WorktopDropManifestInput {
    pub worktop: InternalAddress,
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
#[sbor(transparent)]
pub struct OwnedWorktop(pub Own);

impl Describe<ScryptoCustomTypeKind> for OwnedWorktop {
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1("OwnedWorktop".as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names("OwnedWorktop"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(Some(RESOURCE_PACKAGE), WORKTOP_BLUEPRINT.to_string()),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

pub type WorktopDropOutput = ();

pub const WORKTOP_PUT_IDENT: &str = "Worktop_put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopPutInput {
    pub bucket: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct WorktopPutManifestInput {
    pub bucket: ManifestBucket,
}

pub type WorktopPutOutput = ();

pub const WORKTOP_TAKE_IDENT: &str = "Worktop_take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct WorktopTakeInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub type WorktopTakeManifestInput = WorktopTakeInput;

pub type WorktopTakeOutput = Bucket;

pub const WORKTOP_TAKE_NON_FUNGIBLES_IDENT: &str = "Worktop_take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct WorktopTakeNonFungiblesInput {
    pub ids: IndexSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub type WorktopTakeNonFungiblesManifestInput = WorktopTakeNonFungiblesInput;

pub type WorktopTakeNonFungiblesOutput = Bucket;

pub const WORKTOP_TAKE_ALL_IDENT: &str = "Worktop_take_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct WorktopTakeAllInput {
    pub resource_address: ResourceAddress,
}

pub type WorktopTakeAllManifestInput = WorktopTakeAllInput;

pub type WorktopTakeAllOutput = Bucket;

pub const WORKTOP_ASSERT_CONTAINS_IDENT: &str = "Worktop_assert_contains";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct WorktopAssertContainsInput {
    pub resource_address: ResourceAddress,
}

pub type WorktopAssertContainsManifestInput = WorktopAssertContainsInput;

pub type WorktopAssertContainsOutput = ();

pub const WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT: &str = "Worktop_assert_contains_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct WorktopAssertContainsAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type WorktopAssertContainsAmountManifestInput = WorktopAssertContainsAmountInput;

pub type WorktopAssertContainsAmountOutput = ();

pub const WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT: &str =
    "Worktop_assert_contains_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct WorktopAssertContainsNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: IndexSet<NonFungibleLocalId>,
}

pub type WorktopAssertContainsNonFungiblesManifestInput = WorktopAssertContainsNonFungiblesInput;

pub type WorktopAssertContainsNonFungiblesOutput = ();

pub const WORKTOP_ASSERT_RESOURCES_INCLUDE_IDENT: &str = "Worktop_assert_resources_include";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopAssertResourcesIncludeInput {
    pub constraints: ManifestResourceConstraints,
}

pub type WorktopAssertResourcesIncludeOutput = ();

pub const WORKTOP_ASSERT_RESOURCES_ONLY_IDENT: &str = "Worktop_assert_resources_only";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopAssertResourcesOnlyInput {
    pub constraints: ManifestResourceConstraints,
}

pub type WorktopAssertResourcesOnlyOutput = ();

pub const WORKTOP_DRAIN_IDENT: &str = "Worktop_drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct WorktopDrainInput {}

pub type WorktopDrainManifestInput = WorktopDrainInput;

pub type WorktopDrainOutput = Vec<Bucket>;
