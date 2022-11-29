use radix_engine_interface::api::api::{EngineApi, SysNativeInvokable};
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

pub trait SysVault {
    fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<VaultGetAmountInvocation, E>;
}

impl SysVault for Vault {
    fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<VaultGetAmountInvocation, E>,
    {
        sys_calls.sys_invoke(VaultGetAmountInvocation { receiver: self.0 })
    }
}
