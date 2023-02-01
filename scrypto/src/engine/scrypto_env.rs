use crate::engine::wasm_api::*;
use radix_engine_interface::api::component::ComponentInfoSubstate;
use radix_engine_interface::api::package::PackageInfoSubstate;
use radix_engine_interface::api::{types::*, ClientNativeInvokeApi};
use radix_engine_interface::api::{
    ClientActorApi, ClientComponentApi, ClientNodeApi, ClientPackageApi, ClientSubstateApi,
};
use radix_engine_interface::blueprints::resource::AccessRules;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use sbor::rust::collections::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

#[derive(Debug, Categorize, Encode, Decode)]
pub enum ClientApiError {
    DecodeError(DecodeError),
}

pub struct ScryptoEnv;

impl ClientComponentApi<ClientApiError> for ScryptoEnv {
    fn instantiate_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        access_rules_chain: Vec<AccessRules>,
        royalty_config: RoyaltyConfig,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, ClientApiError> {
        let app_states = scrypto_encode(&app_states).unwrap();
        let access_rules_chain = scrypto_encode(&access_rules_chain).unwrap();
        let royalty_config = scrypto_encode(&royalty_config).unwrap();
        let metadata = scrypto_encode(&metadata).unwrap();

        let bytes = copy_buffer(unsafe {
            instantiate_component(
                blueprint_ident.as_ptr(),
                blueprint_ident.len(),
                app_states.as_ptr(),
                app_states.len(),
                access_rules_chain.as_ptr(),
                access_rules_chain.len(),
                royalty_config.as_ptr(),
                royalty_config.len(),
                metadata.as_ptr(),
                metadata.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize_component(
        &mut self,
        component_id: ComponentId,
    ) -> Result<ComponentAddress, ClientApiError> {
        let component_id = scrypto_encode(&component_id).unwrap();

        let bytes =
            copy_buffer(unsafe { globalize_component(component_id.as_ptr(), component_id.len()) });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn call_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let receiver = scrypto_encode(&receiver).unwrap();

        let return_data = copy_buffer(unsafe {
            call_method(
                receiver.as_ptr(),
                receiver.len(),
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
    }

    fn get_type_info(
        &mut self,
        component_id: ComponentId,
    ) -> Result<(PackageAddress, String), ClientApiError> {
        let component_node_id = RENodeId::Component(component_id);
        let handle = self.sys_lock_substate(
            component_node_id,
            SubstateOffset::Component(ComponentOffset::Info),
            true,
        )?;
        let substate = self.sys_read_substate(handle)?;
        let info: ComponentInfoSubstate = scrypto_decode(&substate).unwrap();
        let package_address = info.package_address.clone();
        let blueprint_ident = info.blueprint_name.clone();
        self.sys_drop_lock(handle)?;
        Ok((package_address, blueprint_ident))
    }
}

impl ClientPackageApi<ClientApiError> for ScryptoEnv {
    fn instantiate_package(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, scrypto_abi::BlueprintAbi>,
        access_rules_chain: Vec<AccessRules>,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    ) -> Result<PackageAddress, ClientApiError> {
        let abi = scrypto_encode(&abi).unwrap();
        let access_rules_chain = scrypto_encode(&access_rules_chain).unwrap();
        let royalty_config = scrypto_encode(&royalty_config).unwrap();
        let metadata = scrypto_encode(&metadata).unwrap();

        let bytes = copy_buffer(unsafe {
            instantiate_package(
                code.as_ptr(),
                code.len(),
                abi.as_ptr(),
                abi.len(),
                access_rules_chain.as_ptr(),
                access_rules_chain.len(),
                royalty_config.as_ptr(),
                royalty_config.len(),
                metadata.as_ptr(),
                metadata.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn get_code(&mut self, package_address: PackageAddress) -> Result<PackageCode, ClientApiError> {
        let package_global = RENodeId::Global(GlobalAddress::Package(package_address));
        let handle = self.sys_lock_substate(
            package_global,
            SubstateOffset::Package(PackageOffset::Info),
            false,
        )?;
        let substate = self.sys_read_substate(handle)?;
        let package: PackageInfoSubstate =
            scrypto_decode(&substate).map_err(ClientApiError::DecodeError)?;
        self.sys_drop_lock(handle)?;
        Ok(PackageCode::Wasm(package.code))
    }

    fn get_abi(
        &mut self,
        package_address: PackageAddress,
    ) -> Result<BTreeMap<String, scrypto_abi::BlueprintAbi>, ClientApiError> {
        let package_global = RENodeId::Global(GlobalAddress::Package(package_address));
        let handle = self.sys_lock_substate(
            package_global,
            SubstateOffset::Package(PackageOffset::Info),
            false,
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

impl ClientNativeInvokeApi<ClientApiError> for ScryptoEnv {
    fn call_native<N: SerializableInvocation>(
        &mut self,
        invocation: N,
    ) -> Result<N::Output, ClientApiError> {
        let fn_identifier = match invocation.fn_identifier() {
            FnIdentifier::Scrypto(_) => {
                panic!(
                    "Please use `call_method` and `call_function` instead for Scrypto invocation"
                )
            }
            FnIdentifier::Native(ident) => ident,
        };

        let invocation = scrypto_encode(&invocation).unwrap();
        let output = self.call_native_raw(fn_identifier, invocation)?;
        scrypto_decode(&output).map_err(ClientApiError::DecodeError)
    }

    fn call_native_raw(
        &mut self,
        fn_identifier: NativeFn,
        invocation: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let fn_identifier = scrypto_encode(&fn_identifier).unwrap();
        let return_data = copy_buffer(unsafe {
            call_native(
                fn_identifier.as_ptr(),
                fn_identifier.len(),
                invocation.as_ptr(),
                invocation.len(),
            )
        });

        scrypto_decode(&return_data).map_err(ClientApiError::DecodeError)
    }
}

impl ClientNodeApi<ClientApiError> for ScryptoEnv {
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();

        unsafe { drop_node(node_id.as_ptr(), node_id.len()) };

        Ok(())
    }

    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, ClientApiError> {
        let node_ids = copy_buffer(unsafe { get_visible_nodes() });

        scrypto_decode(&node_ids).map_err(ClientApiError::DecodeError)
    }
}

impl ClientSubstateApi<ClientApiError> for ScryptoEnv {
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let offset = scrypto_encode(&offset).unwrap();

        let handle = unsafe {
            lock_substate(
                node_id.as_ptr(),
                node_id.len(),
                offset.as_ptr(),
                offset.len(),
                mutable,
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
    fn fn_identifier(&mut self) -> Result<FnIdentifier, ClientApiError> {
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
