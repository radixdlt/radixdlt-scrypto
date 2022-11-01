use crate::engine::utils::WorktopMethodInvocation;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::engine::api::ScryptoNativeInvocation;
use scrypto::engine::utils::{NativeFnInvocation, NativeMethodInvocation};
use scrypto::math::Decimal;
use scrypto::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopPutInvocation {
    pub bucket: Bucket,
}

impl Into<NativeFnInvocation> for WorktopPutInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::Put(self),
        ))
    }
}

impl ScryptoNativeInvocation for WorktopPutInvocation {
    type Output = ();
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAmountInvocation {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Into<NativeFnInvocation> for WorktopTakeAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeAmount(self),
        ))
    }
}

impl ScryptoNativeInvocation for WorktopTakeAmountInvocation {
    type Output = Bucket;
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeNonFungiblesInvocation {
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl Into<NativeFnInvocation> for WorktopTakeNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeNonFungibles(self),
        ))
    }
}

impl ScryptoNativeInvocation for WorktopTakeNonFungiblesInvocation {
    type Output = Bucket;
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAllInvocation {
    pub resource_address: ResourceAddress,
}

impl ScryptoNativeInvocation for WorktopTakeAllInvocation {
    type Output = Bucket;
}

impl Into<NativeFnInvocation> for WorktopTakeAllInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeAll(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsInvocation {
    pub resource_address: ResourceAddress,
}

impl Into<NativeFnInvocation> for WorktopAssertContainsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContains(self),
        ))
    }
}

impl ScryptoNativeInvocation for WorktopAssertContainsInvocation {
    type Output = ();
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsAmountInvocation {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl Into<NativeFnInvocation> for WorktopAssertContainsAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContainsAmount(self),
        ))
    }
}

impl ScryptoNativeInvocation for WorktopAssertContainsAmountInvocation {
    type Output = ();
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsNonFungiblesInvocation {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleId>,
}

impl Into<NativeFnInvocation> for WorktopAssertContainsNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContainsNonFungibles(self),
        ))
    }
}

impl ScryptoNativeInvocation for WorktopAssertContainsNonFungiblesInvocation {
    type Output = ();
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopDrainInvocation {}

impl ScryptoNativeInvocation for WorktopDrainInvocation {
    type Output = Vec<Bucket>;
}

impl Into<NativeFnInvocation> for WorktopDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::Drain(self),
        ))
    }
}
