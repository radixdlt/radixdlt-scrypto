use crate::native_env_native_fn;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

pub struct Worktop;

impl Worktop {
    native_env_native_fn! {
        pub fn sys_put(bucket: Bucket) -> () {
            WorktopPutInvocation {
                bucket
            }
        }
    }

    native_env_native_fn! {
        pub fn sys_take_amount(resource_address: ResourceAddress, amount: Decimal) -> Bucket {
            WorktopTakeAmountInvocation {
                resource_address,
                amount,
            }
        }
    }

    native_env_native_fn! {
        pub fn sys_take_all(resource_address: ResourceAddress) -> Bucket {
            WorktopTakeAllInvocation {
                resource_address,
            }
        }
    }

    native_env_native_fn! {
        pub fn sys_take_non_fungibles(resource_address: ResourceAddress, ids: BTreeSet<NonFungibleLocalId>) -> Bucket {
            WorktopTakeNonFungiblesInvocation {
                resource_address, ids,
            }
        }
    }

    native_env_native_fn! {
        pub fn sys_assert_contains(resource_address: ResourceAddress) -> () {
            WorktopAssertContainsInvocation {
                resource_address,
            }
        }
    }

    native_env_native_fn! {
        pub fn sys_assert_contains_amount(resource_address: ResourceAddress, amount: Decimal) -> () {
            WorktopAssertContainsAmountInvocation {
                resource_address, amount,
            }
        }
    }

    native_env_native_fn! {
        pub fn sys_assert_contains_non_fungibles(resource_address: ResourceAddress, ids: BTreeSet<NonFungibleLocalId>) -> () {
            WorktopAssertContainsNonFungiblesInvocation {
                resource_address, ids,
            }
        }
    }

    native_env_native_fn! {
        pub fn sys_drain() -> Vec<Bucket> {
            WorktopDrainInvocation {}
        }
    }
}
