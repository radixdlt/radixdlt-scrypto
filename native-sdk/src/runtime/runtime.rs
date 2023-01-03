use radix_engine_interface::api::api::{EngineApi, Invokable, InvokableModel};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::constants::EPOCH_MANAGER;
use radix_engine_interface::data::{ScryptoDecode, ScryptoTypeId};
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn sys_current_epoch<Y, E>(api: &mut Y) -> Result<u64, E>
    where
        Y: Invokable<EpochManagerGetCurrentEpochInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode,
    {
        api.invoke(EpochManagerGetCurrentEpochInvocation {
            receiver: EPOCH_MANAGER,
        })
    }

    pub fn generate_uuid<Y, E>(api: &mut Y) -> Result<u128, E>
    where
        Y: EngineApi<E> + InvokableModel<E>,
        E: Debug + ScryptoDecode,
    {
        let visible_node_ids = api.sys_get_visible_nodes()?;
        let node_id = visible_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::TransactionRuntime(..)))
            .expect("TransactionHash does not exist");

        api.invoke(TransactionRuntimeGenerateUuidInvocation {
            receiver: node_id.into(),
        })
    }
}
