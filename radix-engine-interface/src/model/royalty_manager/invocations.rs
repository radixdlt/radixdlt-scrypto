use crate::math::Decimal;
use sbor::rust::fmt::Debug;

use crate::api::{api::*, wasm_input::*};
use crate::model::*;
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct RoyaltyManagerPutInvocation {
    pub bucket: Bucket,
}

impl SysInvocation for RoyaltyManagerPutInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for RoyaltyManagerPutInvocation {}

impl Into<NativeFnInvocation> for RoyaltyManagerPutInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::RoyaltyManager(
            RoyaltyManagerMethodInvocation::Put(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct RoyaltyManagerTakeInvocation {
    pub amount: Decimal,
}

impl SysInvocation for RoyaltyManagerTakeInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for RoyaltyManagerTakeInvocation {}

impl Into<NativeFnInvocation> for RoyaltyManagerTakeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::RoyaltyManager(
            RoyaltyManagerMethodInvocation::Take(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct RoyaltyManagerDrainInvocation {
    pub receiver: SystemAddress,
}

impl SysInvocation for RoyaltyManagerDrainInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for RoyaltyManagerDrainInvocation {}

impl Into<NativeFnInvocation> for RoyaltyManagerDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::RoyaltyManager(
            RoyaltyManagerMethodInvocation::Drain(self),
        ))
    }
}
