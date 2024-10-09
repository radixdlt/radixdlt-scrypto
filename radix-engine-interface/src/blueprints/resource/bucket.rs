use super::Proof;
use super::WithdrawStrategy;
use crate::internal_prelude::*;
use radix_common::data::scrypto::model::Own;
use radix_common::data::scrypto::ScryptoCustomTypeKind;
use radix_common::data::scrypto::ScryptoCustomValueKind;
use radix_common::data::scrypto::*;
use radix_common::math::*;
use radix_common::types::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const BUCKET_TAKE_IDENT: &str = "take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BucketTakeInput {
    pub amount: Decimal,
}

pub type BucketTakeManifestInput = BucketTakeInput;

pub type BucketTakeOutput = Bucket;

pub const BUCKET_TAKE_ADVANCED_IDENT: &str = "take_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BucketTakeAdvancedInput {
    pub amount: Decimal,
    pub withdraw_strategy: WithdrawStrategy,
}

pub type BucketTakeAdvancedManifestInput = BucketTakeAdvancedInput;

pub type BucketTakeAdvancedOutput = Bucket;

pub const BUCKET_PUT_IDENT: &str = "put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct BucketPutInput {
    pub bucket: Bucket,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct BucketPutManifestInput {
    pub bucket: ManifestBucket,
}

pub type BucketPutOutput = ();

pub const BUCKET_GET_AMOUNT_IDENT: &str = "get_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BucketGetAmountInput {}

pub type BucketGetAmountManifestInput = BucketGetAmountInput;

pub type BucketGetAmountOutput = Decimal;

pub const BUCKET_GET_RESOURCE_ADDRESS_IDENT: &str = "get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BucketGetResourceAddressInput {}

pub type BucketGetResourceAddressManifestInput = BucketGetResourceAddressInput;

pub type BucketGetResourceAddressOutput = ResourceAddress;

pub const BUCKET_CREATE_PROOF_OF_ALL_IDENT: &str = "create_proof_of_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BucketCreateProofOfAllInput {}

pub type BucketCreateProofOfAllManifestInput = BucketCreateProofOfAllInput;

pub type BucketCreateProofOfAllOutput = Proof;

//========
// Stub
//========

#[derive(Debug, PartialEq, Eq, Hash)]
#[must_use]
pub struct Bucket(pub Own);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
#[must_use]
pub struct FungibleBucket(pub Bucket);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
#[must_use]
pub struct NonFungibleBucket(pub Bucket);

impl AsRef<Bucket> for FungibleBucket {
    fn as_ref(&self) -> &Bucket {
        &self.0
    }
}

impl From<FungibleBucket> for Bucket {
    fn from(value: FungibleBucket) -> Self {
        value.0
    }
}

impl AsRef<Bucket> for NonFungibleBucket {
    fn as_ref(&self) -> &Bucket {
        &self.0
    }
}

impl From<NonFungibleBucket> for Bucket {
    fn from(value: NonFungibleBucket) -> Self {
        value.0
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Bucket {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Bucket {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Bucket {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for Bucket {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_BUCKET_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::own_bucket_type_data()
    }
}

impl Describe<ScryptoCustomTypeKind> for FungibleBucket {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_FUNGIBLE_BUCKET_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::own_fungible_bucket_type_data()
    }
}

impl Describe<ScryptoCustomTypeKind> for NonFungibleBucket {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_NON_FUNGIBLE_BUCKET_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::own_non_fungible_bucket_type_data()
    }
}
