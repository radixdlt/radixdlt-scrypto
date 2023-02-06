use crate::api::types::*;
use crate::blueprints::resource::Bucket;
use crate::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct FeeReserveLockFeeInvocation {
    // FIXME: protect this method for vault::lock_fee actor only!
    pub receiver: FeeReserveId, // Not in use
    pub vault_id: VaultId,
    pub bucket: Bucket,
    pub contingent: bool,
}

impl Clone for FeeReserveLockFeeInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            vault_id: self.vault_id,
            bucket: Bucket(self.bucket.0),
            contingent: self.contingent,
        }
    }
}

impl Invocation for FeeReserveLockFeeInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::FeeReserve(FeeReserveFn::LockFee))
    }
}

impl SerializableInvocation for FeeReserveLockFeeInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::FeeReserve(FeeReserveFn::LockFee)
    }
}

impl Into<CallTableInvocation> for FeeReserveLockFeeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::FeeReserve(FeeReserveInvocation::LockFee(self)).into()
    }
}
