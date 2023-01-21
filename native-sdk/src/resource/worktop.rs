use crate::sys_env_native_fn;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::*;

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
        pub fn sys_take_non_fungibles(resource_address: ResourceAddress, ids: BTreeSet<NonFungibleLocalId>) -> Bucket {
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
        pub fn sys_assert_contains_non_fungibles(resource_address: ResourceAddress, ids: BTreeSet<NonFungibleLocalId>) -> () {
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
