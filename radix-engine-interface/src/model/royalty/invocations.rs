use crate::math::Decimal;
use sbor::rust::fmt::Debug;

use crate::api::{api::*, types::*, wasm_input::*};
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct RoyaltyReservePutInvocation {
    pub receiver: RoyaltyReserveId,
    pub bucket: Bucket,
}

impl SysInvocation for RoyaltyReservePutInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for RoyaltyReservePutInvocation {}

impl Into<NativeFnInvocation> for RoyaltyReservePutInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::RoyaltyReserve(
            RoyaltyReserveMethodInvocation::Put(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct RoyaltyReserveTakeInvocation {
    pub receiver: RoyaltyReserveId,
    pub amount: Decimal,
}

impl SysInvocation for RoyaltyReserveTakeInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for RoyaltyReserveTakeInvocation {}

impl Into<NativeFnInvocation> for RoyaltyReserveTakeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::RoyaltyReserve(
            RoyaltyReserveMethodInvocation::Take(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct RoyaltyReserveDrainInvocation {
    pub receiver: RoyaltyReserveId,
}

impl SysInvocation for RoyaltyReserveDrainInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for RoyaltyReserveDrainInvocation {}

impl Into<NativeFnInvocation> for RoyaltyReserveDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::RoyaltyReserve(
            RoyaltyReserveMethodInvocation::Drain(self),
        ))
    }
}
