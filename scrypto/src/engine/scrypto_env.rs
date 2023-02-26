use crate::engine::wasm_api::*;
use radix_engine_interface::api::package::{PackageCodeSubstate, PackageInfoSubstate};
use radix_engine_interface::api::{types::*, LockFlags};
use radix_engine_interface::api::{
    ClientActorApi, ClientComponentApi, ClientNodeApi, ClientPackageApi, ClientSubstateApi,
};
use radix_engine_interface::blueprints::resource::AccessRules;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use sbor::rust::collections::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use scrypto_abi::BlueprintAbi;

#[derive(Debug, Sbor)]
pub enum ClientApiError {
    DecodeError(DecodeError),
}

pub struct ScryptoEnv;

impl ClientComponentApi<ClientApiError> for ScryptoEnv {
    fn new_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        royalty_config: RoyaltyConfig,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, ClientApiError> {
        let app_states = scrypto_encode(&app_states).unwrap();
        let royalty_config = scrypto_encode(&royalty_config).unwrap();
        let metadata = scrypto_encode(&metadata).unwrap();

        let bytes = copy_buffer(unsafe {
            new_component(
                blueprint_ident.as_ptr(),
                blueprint_ident.len(),
                app_states.as_ptr(),
                app_states.len(),
                royalty_config.as_ptr(),
                royalty_config.len(),
                metadata.as_ptr(),
                metadata.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize(
        &mut self,
        node_id: RENodeId,
        access_rules: AccessRules,
    ) -> Result<ComponentAddress, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let access_rules = scrypto_encode(&access_rules).unwrap();

        let bytes = copy_buffer(unsafe {
            globalize_component(
                node_id.as_ptr(),
                node_id.len(),
                access_rules.as_ptr(),
                access_rules.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize_with_address(
        &mut self,
        node_id: RENodeId,
        access_rules: AccessRules,
        address: Address,
    ) -> Result<ComponentAddress, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let access_rules = scrypto_encode(&access_rules).unwrap();
        let address = scrypto_encode(&address).unwrap();

        let bytes = copy_buffer(unsafe {
            globalize_with_address(
                node_id.as_ptr(),
                node_id.len(),
                access_rules.as_ptr(),
                access_rules.len(),
                address.as_ptr(),
                address.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn call_method(
        &mut self,
        receiver: RENodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        self.call_module_method(receiver, NodeModuleId::SELF, method_name, args)
    }

    fn call_module_method(
        &mut self,
        receiver: RENodeId,
        node_module_id: NodeModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let receiver = scrypto_encode(&receiver).unwrap();

        let return_data = copy_buffer(unsafe {
            call_method(
                receiver.as_ptr(),
                receiver.len(),
                node_module_id.id(),
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
    }

    fn get_component_type_info(
        &mut self,
        node_id: RENodeId,
    ) -> Result<(PackageAddress, String), ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();

        let bytes =
            copy_buffer(unsafe { get_component_type_info(node_id.as_ptr(), node_id.len()) });

        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn new_key_value_store(&mut self) -> Result<KeyValueStoreId, ClientApiError> {
        let bytes = copy_buffer(unsafe { new_key_value_store() });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }
}

impl ClientPackageApi<ClientApiError> for ScryptoEnv {
    fn new_package(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        access_rules: AccessRules,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    ) -> Result<PackageAddress, ClientApiError> {
        let abi = scrypto_encode(&abi).unwrap();
        let access_rules = scrypto_encode(&access_rules).unwrap();
        let royalty_config = scrypto_encode(&royalty_config).unwrap();
        let metadata = scrypto_encode(&metadata).unwrap();

        let bytes = copy_buffer(unsafe {
            new_package(
                code.as_ptr(),
                code.len(),
                abi.as_ptr(),
                abi.len(),
                access_rules.as_ptr(),
                access_rules.len(),
                royalty_config.as_ptr(),
                royalty_config.len(),
                metadata.as_ptr(),
                metadata.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn get_code(&mut self, package_address: PackageAddress) -> Result<PackageCode, ClientApiError> {
        let package_global = RENodeId::GlobalPackage(package_address);
        let handle = self.sys_lock_substate(
            package_global,
            SubstateOffset::Package(PackageOffset::Code),
            LockFlags::read_only(),
        )?;
        let substate = self.sys_read_substate(handle)?;
        let package: PackageCodeSubstate =
            scrypto_decode(&substate).map_err(ClientApiError::DecodeError)?;
        self.sys_drop_lock(handle)?;
        Ok(PackageCode::Wasm(package.code))
    }

    fn get_abi(
        &mut self,
        package_address: PackageAddress,
    ) -> Result<BTreeMap<String, scrypto_abi::BlueprintAbi>, ClientApiError> {
        let package_global = RENodeId::GlobalPackage(package_address);
        let handle = self.sys_lock_substate(
            package_global,
            SubstateOffset::Package(PackageOffset::Info),
            LockFlags::read_only(),
        )?;
        let substate = self.sys_read_substate(handle)?;
        let package: PackageInfoSubstate =
            scrypto_decode(&substate).map_err(ClientApiError::DecodeError)?;
        self.sys_drop_lock(handle)?;
        Ok(package.blueprint_abis)
    }

    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let package_address = scrypto_encode(&package_address).unwrap();

        let return_data = copy_buffer(unsafe {
            call_function(
                package_address.as_ptr(),
                package_address.len(),
                blueprint_name.as_ptr(),
                blueprint_name.len(),
                function_name.as_ptr(),
                function_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
    }
}

impl ClientNodeApi<ClientApiError> for ScryptoEnv {
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();

        unsafe { drop_node(node_id.as_ptr(), node_id.len()) };

        Ok(())
    }
}

impl ClientSubstateApi<ClientApiError> for ScryptoEnv {
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let offset = scrypto_encode(&offset).unwrap();

        let handle = unsafe {
            lock_substate(
                node_id.as_ptr(),
                node_id.len(),
                offset.as_ptr(),
                offset.len(),
                flags.bits(),
            )
        };

        Ok(handle)
    }

    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, ClientApiError> {
        let substate = copy_buffer(unsafe { read_substate(lock_handle) });

        Ok(substate)
    }

    fn sys_write_substate(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { write_substate(lock_handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), ClientApiError> {
        unsafe { drop_lock(lock_handle) };

        Ok(())
    }
}

impl ClientActorApi<ClientApiError> for ScryptoEnv {
    fn get_fn_identifier(&mut self) -> Result<FnIdentifier, ClientApiError> {
        let actor = copy_buffer(unsafe { get_actor() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }
}

#[macro_export]
macro_rules! scrypto_env_native_fn {
    ($($vis:vis $fn:ident $fn_name:ident ($($args:tt)*) -> $rtn:ty { $arg:expr })*) => {
        $(
            $vis $fn $fn_name ($($args)*) -> $rtn {
                let mut env = crate::engine::scrypto_env::ScryptoEnv;
                env.call_native($arg).unwrap()
            }
        )+
    };
}
