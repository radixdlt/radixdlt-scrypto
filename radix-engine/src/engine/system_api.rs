use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::SNodeRef;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::*;
use crate::model::*;
use crate::wasm::*;

pub trait SystemApi<W, I>
where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn wasm_engine(&mut self) -> &mut W;

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        call_data: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible>;

    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    );

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError>;

    fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    );

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError>;

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError>;

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError>;

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError>;

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError>;

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress;

    fn create_package(&mut self, package: Package) -> PackageAddress;

    fn create_component(&mut self, component: Component) -> Result<ComponentAddress, RuntimeError>;

    fn read_component_state(&mut self, addr: ComponentAddress) -> Result<Vec<u8>, RuntimeError>;

    fn write_component_state(
        &mut self,
        addr: ComponentAddress,
        state: Vec<u8>,
    ) -> Result<(), RuntimeError>;

    fn get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError>;

    fn create_lazy_map(&mut self) -> LazyMapId;

    fn read_lazy_map_entry(
        &mut self,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, RuntimeError>;

    fn write_lazy_map_entry(
        &mut self,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), RuntimeError>;

    fn get_epoch(&mut self) -> u64;

    fn get_transaction_hash(&mut self) -> Hash;

    fn generate_uuid(&mut self) -> u128;

    fn user_log(&mut self, level: Level, message: String);

    fn sys_log(&self, level: Level, message: String);
}
