use crate::api::types::{AccessRuleKey, MetadataMethod, NativeFn, NativeMethod, PackageMethod};
use crate::scrypto;
use sbor::*;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum PackageMethodAuthKey {
    SetMetadata,
    GetMetadata,
    SetRoyaltyConfig,
    ClaimRoyalty,
}

impl From<PackageMethodAuthKey> for AccessRuleKey {
    fn from(auth_key: PackageMethodAuthKey) -> Self {
        match auth_key {
            PackageMethodAuthKey::SetMetadata => AccessRuleKey::Native(NativeFn::Method(
                NativeMethod::Metadata(MetadataMethod::Set),
            )),
            PackageMethodAuthKey::GetMetadata => AccessRuleKey::Native(NativeFn::Method(
                NativeMethod::Metadata(MetadataMethod::Get),
            )),
            PackageMethodAuthKey::SetRoyaltyConfig => AccessRuleKey::Native(NativeFn::Method(
                NativeMethod::Package(PackageMethod::SetRoyaltyConfig),
            )),
            PackageMethodAuthKey::ClaimRoyalty => AccessRuleKey::Native(NativeFn::Method(
                NativeMethod::Package(PackageMethod::ClaimRoyalty),
            )),
        }
    }
}
