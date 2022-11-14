use radix_engine_lib::component::PackageAddress;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::engine::api::{ScryptoNativeInvocation, SysInvocation};
use utils::crypto::Blob;

use crate::core::*;
use crate::engine::scrypto_env::{
    NativeFnInvocation, NativeFunctionInvocation, PackageFunctionInvocation,
};

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PackagePublishInvocation {
    pub code: Blob,
    pub abi: Blob,
}

impl SysInvocation for PackagePublishInvocation {
    type Output = PackageAddress;
}

impl ScryptoNativeInvocation for PackagePublishInvocation {}

impl Into<NativeFnInvocation> for PackagePublishInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Package(
            PackageFunctionInvocation::Publish(self),
        ))
    }
}

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: Decode>(&self, blueprint_name: &str, function: &str, args: Vec<u8>) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }
}