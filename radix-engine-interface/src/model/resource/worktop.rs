use crate::api::api::Invocation;
use crate::math::Decimal;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopPutInvocation {
    pub bucket: Bucket,
}

impl Invocation for WorktopPutInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopPutInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for WorktopPutInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::Put(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopTakeAmountInvocation {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopTakeAmountInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for WorktopTakeAmountInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<NativeFnInvocation> for WorktopTakeAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeAmount(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopTakeNonFungiblesInvocation {
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopTakeNonFungiblesInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for WorktopTakeNonFungiblesInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<NativeFnInvocation> for WorktopTakeNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeNonFungibles(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopTakeAllInvocation {
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopTakeAllInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for WorktopTakeAllInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<NativeFnInvocation> for WorktopTakeAllInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeAll(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopAssertContainsInvocation {
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopAssertContainsInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopAssertContainsInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for WorktopAssertContainsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContains(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopAssertContainsAmountInvocation {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}
impl Invocation for WorktopAssertContainsAmountInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopAssertContainsAmountInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for WorktopAssertContainsAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContainsAmount(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopAssertContainsNonFungiblesInvocation {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleId>,
}

impl Invocation for WorktopAssertContainsNonFungiblesInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopAssertContainsNonFungiblesInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for WorktopAssertContainsNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContainsNonFungibles(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WorktopDrainInvocation {}

impl Invocation for WorktopDrainInvocation {
    type Output = Vec<Bucket>;
}

impl ScryptoNativeInvocation for WorktopDrainInvocation {
    type ScryptoOutput = Vec<Bucket>;
}

impl Into<NativeFnInvocation> for WorktopDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::Drain(self),
        ))
    }
}
