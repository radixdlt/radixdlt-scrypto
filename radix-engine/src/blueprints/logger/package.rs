use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::logger::*;

pub struct LoggerNativePackage;
impl LoggerNativePackage {
    pub(crate) fn log_message<Y>(
        level: Level,
        message: String,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelModuleApi<RuntimeError>,
    {
        api.kernel_get_module_state().logger.add_log(level, message);

        Ok(())
    }
}
