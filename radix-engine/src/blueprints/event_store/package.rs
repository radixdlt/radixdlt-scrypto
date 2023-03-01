use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::actor::{Actor, ActorIdentifier};
use crate::kernel::kernel_api::{KernelInternalApi, KernelModuleApi};
use crate::system::events::EventError;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use radix_engine_interface::abi::LegacyDescribe;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientUnsafeApi;
use radix_engine_interface::events::EventTypeIdentifier;

pub struct EventStoreNativePackage;
impl EventStoreNativePackage {
    pub(crate) fn emit_event<Y, T>(event: T, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelModuleApi<RuntimeError> + ClientUnsafeApi<RuntimeError> + KernelInternalApi,
        T: ScryptoEncode + LegacyDescribe,
    {
        let schema_hash = scrypto_encode(&T::describe())
            .map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::EventError(
                    EventError::FailedToSborEncodeEventSchema,
                ))
            })
            .map(|encoded| hash(encoded))?;
        let event_data = scrypto_encode(&event).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::EventError(
                EventError::FailedToSborEncodeEvent,
            ))
        })?;

        Self::emit_raw_event(schema_hash, event_data, api)
    }

    pub(crate) fn emit_raw_event<Y>(
        schema_hash: Hash,
        event_data: Vec<u8>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelModuleApi<RuntimeError> + ClientUnsafeApi<RuntimeError> + KernelInternalApi,
    {
        // Costing event emission.
        api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

        // Construct the event type identifier based on the current actor
        let event_type_id = match api.kernel_get_current_actor() {
            Some(Actor {
                identifier: ActorIdentifier::Method(MethodIdentifier(node_id, node_module_id, ..)),
                ..
            }) => Ok(EventTypeIdentifier(node_id, node_module_id, schema_hash)),
            Some(Actor {
                identifier:
                    ActorIdentifier::Function(FnIdentifier {
                        package_address, ..
                    }),
                ..
            }) => Ok(EventTypeIdentifier(
                RENodeId::GlobalPackage(package_address),
                NodeModuleId::SELF,
                schema_hash,
            )),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::EventError(EventError::InvalidActor),
            )),
        }?;

        // TODO: Validate that the event schema matches that given by the event schema hash.
        // Need to wait for David's PR for schema validation and move away from LegacyDescribe
        // over to new Describe.

        // NOTE: We need to ensure that the event being emitted is an SBOR struct or an enum,
        // this is not done here, this should be done at event registration time. Thus, if the
        // event has been successfully registered, it can be emitted (from a schema POV).

        // Adding the event to the event store
        api.kernel_get_module_state()
            .events
            .add_event(event_type_id, event_data);

        Ok(())
    }
}
