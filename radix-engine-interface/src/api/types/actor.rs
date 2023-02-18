use crate::api::package::PackageAddress;
use crate::api::types::*;
use crate::*;
use sbor::rust::string::String;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum PackageIdentifier {
    Scrypto(PackageAddress),
    Native(NativePackage),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum NativePackage {
    Auth,
    Package,
    TransactionProcessor,
    Root,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum FnIdentifier {
    Scrypto(ScryptoFnIdentifier),
    Native(NativeFn),
}

impl From<NativeFn> for FnIdentifier {
    fn from(native_fn: NativeFn) -> Self {
        FnIdentifier::Native(native_fn)
    }
}

impl FnIdentifier {
    pub fn package_identifier(&self) -> PackageIdentifier {
        match self {
            FnIdentifier::Scrypto(identifier) => {
                PackageIdentifier::Scrypto(identifier.package_address)
            }
            FnIdentifier::Native(identifier) => PackageIdentifier::Native(identifier.package()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ScryptoFnIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub ident: String,
}

impl ScryptoFnIdentifier {
    pub fn new(package_address: PackageAddress, blueprint_name: String, ident: String) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident,
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        &self.package_address
    }

    pub fn blueprint_name(&self) -> &String {
        &self.blueprint_name
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
pub enum NativeFn {
    Package(PackageFn),
    TransactionProcessor(TransactionProcessorFn),
    Root,
}

impl NativeFn {
    pub fn package(&self) -> NativePackage {
        match self {
            NativeFn::Package(..) => NativePackage::Package,
            NativeFn::TransactionProcessor(..) => NativePackage::TransactionProcessor,
            NativeFn::Root => NativePackage::Root,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum PackageFn {
    Publish,
    PublishNative,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionProcessorFn {
    Run,
}
