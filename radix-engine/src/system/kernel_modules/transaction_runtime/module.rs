use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::{
    blueprints::transaction_runtime::TransactionRuntimeSubstate, errors::RuntimeError,
    system::node::RENodeInit,
};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId, RENodeType};
use radix_engine_interface::blueprints::transaction_runtime::TRANSACTION_RUNTIME_BLUEPRINT;
use radix_engine_interface::constants::TRANSACTION_RUNTIME_PACKAGE;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::ScryptoValue;
use sbor::btreemap;

#[derive(Debug, Clone)]
pub struct TransactionRuntimeModule {
    pub tx_hash: Hash,
}

impl KernelModule for TransactionRuntimeModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let hash = api
            .kernel_get_module_state()
            .transaction_runtime
            .tx_hash
            .clone();

        let node_id = api.kernel_allocate_node_id(RENodeType::TransactionRuntime)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::TransactionRuntime(TransactionRuntimeSubstate {
                hash,
                next_id: 0u32,
            }),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: TRANSACTION_RUNTIME_PACKAGE,
                        blueprint_name: TRANSACTION_RUNTIME_BLUEPRINT.to_string(),
                        global: false,
                    })
            ),
        )?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_drop_node(RENodeId::TransactionRuntime)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &Option<Actor>,
        call_frame_update: &mut CallFrameUpdate,
        _args: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::TransactionRuntime);

        Ok(())
    }
}
