use sbor::*;
use sbor::rust::vec::Vec;
use sbor::rust::collections::BTreeSet;
use scrypto::engine::api::SysInvocation;
use scrypto::math::Decimal;
use scrypto::resource::*;
use crate::engine::types::{NativeMethod, WorktopMethod};

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopPutInvocation {
    pub bucket: Bucket,
}

impl SysInvocation for WorktopPutInvocation {
    type Output = ();
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::Put)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAmountInvocation {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for WorktopTakeAmountInvocation {
    type Output = Bucket;
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::TakeAmount)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeNonFungiblesInvocation {
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for WorktopTakeNonFungiblesInvocation {
    type Output = Bucket;
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::TakeNonFungibles)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAllInvocation {
    pub resource_address: ResourceAddress,
}

impl SysInvocation for WorktopTakeAllInvocation {
    type Output = Bucket;
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::TakeAll)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsInvocation {
    pub resource_address: ResourceAddress,
}

impl SysInvocation for WorktopAssertContainsInvocation {
    type Output = ();
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::AssertContains)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsAmountInvocation {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl SysInvocation for WorktopAssertContainsAmountInvocation {
    type Output = ();
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::AssertContainsAmount)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsNonFungiblesInvocation {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleId>,
}

impl SysInvocation for WorktopAssertContainsNonFungiblesInvocation {
    type Output = ();
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::AssertContainsNonFungibles)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopDrainInvocation {}

impl SysInvocation for WorktopDrainInvocation {
    type Output = Vec<Bucket>;
    fn native_method() -> NativeMethod {
        NativeMethod::Worktop(WorktopMethod::Drain)
    }
}