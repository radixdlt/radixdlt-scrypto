use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::SNodeRef;
use scrypto::engine::types::*;
use scrypto::resource::AccessRule;
use scrypto::values::*;

use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::wasm::*;

pub trait SystemApi<W, I>
where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn wasm_engine(&mut self) -> &mut W;

    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter;

    fn fee_table(&self) -> &FeeTable;

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
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

    fn create_package(&mut self, package: ValidatedPackage) -> PackageAddress;

    fn create_component(&mut self, component: Component) -> Result<ComponentAddress, RuntimeError>;

    fn read_component_state(&mut self, addr: ComponentAddress) -> Result<Vec<u8>, RuntimeError>;

    fn write_component_state(
        &mut self,
        addr: ComponentAddress,
        state: ScryptoValue,
    ) -> Result<(), RuntimeError>;

    fn get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError>;

    fn create_kv_store(&mut self) -> KeyValueStoreId;

    fn read_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn write_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: ScryptoValue,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError>;

    fn get_epoch(&mut self) -> u64;

    fn get_transaction_hash(&mut self) -> Hash;

    fn generate_uuid(&mut self) -> u128;

    fn user_log(&mut self, level: Level, message: String);

    fn sys_log(&self, level: Level, message: String);

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;
}
