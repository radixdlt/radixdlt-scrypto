use std::fmt::Debug;
use radix_engine_interface::api::ClientApi;
use crate::sys_env_native_fn;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

pub struct Worktop;

impl Worktop {
    pub fn sys_put<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E>
        where
            Y: ClientApi<E>,
    {
        let _rtn = api.call_method(
            ScryptoReceiver::Worktop,
            WORKTOP_PUT_IDENT,
            scrypto_encode(&WorktopPutInput { bucket }).unwrap()
        )?;

        Ok(())
    }

    pub fn sys_take<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
        where
            Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Worktop,
            WORKTOP_TAKE_IDENT,
            scrypto_encode(&WorktopTakeInput { resource_address, amount }).unwrap()
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_take_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
        where
            Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Worktop,
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&WorktopTakeNonFungiblesInput { resource_address, ids, }).unwrap()
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_take_all<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>
        where
            Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Worktop,
            WORKTOP_TAKE_ALL_IDENT,
            scrypto_encode(&WorktopTakeAllInput { resource_address }).unwrap()
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
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
