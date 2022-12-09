use radix_engine_interface::api::api::{EngineApi, Invokable};
use radix_engine_interface::data::types::Ownership;
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

pub struct Vault(pub Ownership); // native stub

pub trait SysVault {
    fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + Invokable<VaultGetAmountInvocation, E>;
}

impl SysVault for Vault {
    fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + Invokable<VaultGetAmountInvocation, E>,
    {
        sys_calls.invoke(VaultGetAmountInvocation {
            receiver: self.0.vault_id(),
        })
    }
}
