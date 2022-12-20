use radix_engine_interface::api::api::Invokable;
use radix_engine_interface::constants::EPOCH_MANAGER;
use radix_engine_interface::data::{ScryptoDecode, ScryptoTypeId};
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn sys_current_epoch<Y, E>(env: &mut Y) -> Result<u64, E>
    where
        Y: Invokable<EpochManagerGetCurrentEpochInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode,
    {
        env.invoke(EpochManagerGetCurrentEpochInvocation {
            receiver: EPOCH_MANAGER,
        })
    }
}
