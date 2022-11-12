use radix_engine_lib::resource::ResourceAddress;
use crate::engine::scrypto_env::WorktopMethodInvocation;
use crate::sys_env_native_fn;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::engine::api::{ScryptoNativeInvocation, SysInvocation};
use scrypto::engine::scrypto_env::{NativeFnInvocation, NativeMethodInvocation};
use scrypto::math::Decimal;
use scrypto::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopPutInvocation {
    pub bucket: Bucket,
}

impl SysInvocation for WorktopPutInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopPutInvocation {}

impl Into<NativeFnInvocation> for WorktopPutInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::Put(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAmountInvocation {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for WorktopTakeAmountInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for WorktopTakeAmountInvocation {}

impl Into<NativeFnInvocation> for WorktopTakeAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeAmount(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeNonFungiblesInvocation {
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for WorktopTakeNonFungiblesInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for WorktopTakeNonFungiblesInvocation {}

impl Into<NativeFnInvocation> for WorktopTakeNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::TakeNonFungibles(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopTakeAllInvocation {
    pub resource_address: ResourceAddress,
}

impl SysInvocation for WorktopTakeAllInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for WorktopTakeAllInvocation {}

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

impl SysInvocation for WorktopAssertContainsInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopAssertContainsInvocation {}

impl Into<NativeFnInvocation> for WorktopAssertContainsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContains(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsAmountInvocation {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}
impl SysInvocation for WorktopAssertContainsAmountInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopAssertContainsAmountInvocation {}

impl Into<NativeFnInvocation> for WorktopAssertContainsAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContainsAmount(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopAssertContainsNonFungiblesInvocation {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleId>,
}

impl SysInvocation for WorktopAssertContainsNonFungiblesInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for WorktopAssertContainsNonFungiblesInvocation {}

impl Into<NativeFnInvocation> for WorktopAssertContainsNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::AssertContainsNonFungibles(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct WorktopDrainInvocation {}

impl SysInvocation for WorktopDrainInvocation {
    type Output = Vec<Bucket>;
}

impl ScryptoNativeInvocation for WorktopDrainInvocation {}

impl Into<NativeFnInvocation> for WorktopDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Worktop(
            WorktopMethodInvocation::Drain(self),
        ))
    }
}

pub struct Worktop;

impl Worktop {
    sys_env_native_fn! {
        pub fn sys_put(bucket: Bucket) -> () {
            WorktopPutInvocation {
                bucket
            }
        }
    }

    sys_env_native_fn! {
        pub fn sys_take_amount(resource_address: ResourceAddress, amount: Decimal) -> Bucket {
            WorktopTakeAmountInvocation {
                resource_address,
                amount,
            }
        }
    }

    sys_env_native_fn! {
        pub fn sys_take_all(resource_address: ResourceAddress) -> Bucket {
            WorktopTakeAllInvocation {
                resource_address,
            }
        }
    }

    sys_env_native_fn! {
        pub fn sys_take_non_fungibles(resource_address: ResourceAddress, ids: BTreeSet<NonFungibleId>) -> Bucket {
            WorktopTakeNonFungiblesInvocation {
                resource_address, ids,
            }
        }
    }

    sys_env_native_fn! {
        pub fn sys_assert_contains(resource_address: ResourceAddress) -> () {
            WorktopAssertContainsInvocation {
                resource_address,
            }
        }
    }

    sys_env_native_fn! {
        pub fn sys_assert_contains_amount(resource_address: ResourceAddress, amount: Decimal) -> () {
            WorktopAssertContainsAmountInvocation {
                resource_address, amount,
            }
        }
    }

    sys_env_native_fn! {
        pub fn sys_assert_contains_non_fungibles(resource_address: ResourceAddress, ids: BTreeSet<NonFungibleId>) -> () {
            WorktopAssertContainsNonFungiblesInvocation {
                resource_address, ids,
            }
        }
    }

    sys_env_native_fn! {
        pub fn sys_drain() -> Vec<Bucket> {
            WorktopDrainInvocation {}
        }
    }
}
