use core::u32;

use crate::errors::InvokeError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::vm::wasm::*;
use crate::vm::ScryptoVmVersion;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::key_value_store_api::KeyValueStoreDataSchema;
use radix_engine_interface::api::{ActorRefHandle, AttachedModuleId, FieldValue, SystemApi};
use radix_engine_interface::types::ClientCostingEntry;
use radix_engine_interface::types::Level;
use radix_engine_profiling_derive::trace_resources;
use sbor::rust::vec::Vec;

/// A shim between SystemAPI and WASM, with buffer capability.
pub struct ScryptoRuntime<'y, Y: SystemApi<RuntimeError>> {
    api: &'y mut Y,
    buffers: IndexMap<BufferId, Vec<u8>>,
    next_buffer_id: BufferId,
    package_address: PackageAddress,
    export_name: String,
    wasm_execution_units_buffer: u32,
    scrypto_vm_version: ScryptoVmVersion,
    wasm_execution_units_base: u32,
}

impl<'y, Y: SystemApi<RuntimeError>> ScryptoRuntime<'y, Y> {
    pub fn new(
        api: &'y mut Y,
        package_address: PackageAddress,
        export_name: String,
        scrypto_vm_version: ScryptoVmVersion,
    ) -> Self {
        let wasm_execution_units_base = if scrypto_vm_version < ScryptoVmVersion::cuttlefish() {
            0
        } else {
            // Add 28,000 base units to make sure the we do not undercharge for WASM execution,
            // which might lead to system exploitation.
            // This is especially important in corner-cases such as `costing::spin_loop_v2` benchmark.
            // less frequently.
            28000
        };

        ScryptoRuntime {
            api,
            buffers: index_map_new(),
            next_buffer_id: 0,
            package_address,
            export_name,
            wasm_execution_units_buffer: 0,
            scrypto_vm_version,
            wasm_execution_units_base,
        }
    }
    pub fn parse_blueprint_id(
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<(PackageAddress, String), InvokeError<WasmRuntimeError>> {
        let package_address = PackageAddress::try_from(package_address.as_slice())
            .map_err(|_| WasmRuntimeError::InvalidPackageAddress)?;
        let blueprint_name =
            String::from_utf8(blueprint_name).map_err(|_| WasmRuntimeError::InvalidString)?;
        Ok((package_address, blueprint_name))
    }

    #[cold]
    fn consume_wasm_execution_exceeding_buffer(
        &mut self,
        n: u32,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        assert!(n > self.wasm_execution_units_buffer);
        let n_remaining_after_buffer_used = n - self.wasm_execution_units_buffer;
        let amount_to_request =
            n_remaining_after_buffer_used.saturating_add(WASM_EXECUTION_COST_UNITS_BUFFER);

        self.api
            .consume_cost_units(ClientCostingEntry::RunWasmCode {
                package_address: &self.package_address,
                export_name: &self.export_name,
                wasm_execution_units: amount_to_request,
            })
            .map_err(InvokeError::downstream)?;
        self.wasm_execution_units_buffer = amount_to_request - n_remaining_after_buffer_used;
        Ok(())
    }
}

impl<'y, Y: SystemApi<RuntimeError>> WasmRuntime for ScryptoRuntime<'y, Y> {
    fn allocate_buffer(
        &mut self,
        buffer: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        assert!(buffer.len() <= 0xffffffff);

        let max_number_of_buffers = match self.scrypto_vm_version {
            ScryptoVmVersion::V1_0 | ScryptoVmVersion::V1_1 => 32,
            // Practically speaking, there is little gain of keeping multiple buffers open before
            // [multi-value](https://github.com/WebAssembly/multi-value/blob/master/proposals/multi-value/Overview.md) is supported and used.
            // We reduce it to `4` so that the amount of memory that a transaction can consume is reduced, which is beneficial for parallel execution.
            ScryptoVmVersion::V1_2 => 4,
        };
        if self.buffers.len() >= max_number_of_buffers {
            return Err(InvokeError::SelfError(WasmRuntimeError::TooManyBuffers));
        }

        let id = self.next_buffer_id;
        let len = buffer.len();

        self.buffers.insert(id, buffer);
        self.next_buffer_id += 1;

        Ok(Buffer::new(id, len as u32))
    }

    fn buffer_consume(
        &mut self,
        buffer_id: BufferId,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        self.buffers
            .swap_remove(&buffer_id)
            .ok_or(InvokeError::SelfError(WasmRuntimeError::BufferNotFound(
                buffer_id,
            )))
    }

    fn object_call(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(receiver.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let return_data = self.api.call_method(&receiver, ident.as_str(), args)?;

        self.allocate_buffer(return_data)
    }

    fn object_call_module(
        &mut self,
        receiver: Vec<u8>,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(receiver.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let module_id = u8::try_from(module_id)
            .ok()
            .and_then(|x| AttachedModuleId::from_repr(x))
            .ok_or(WasmRuntimeError::InvalidAttachedModuleId(module_id))?;

        let return_data =
            self.api
                .call_module_method(&receiver, module_id, ident.as_str(), args)?;

        self.allocate_buffer(return_data)
    }

    fn object_call_direct(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(receiver.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let return_data = self
            .api
            .call_direct_access_method(&receiver, ident.as_str(), args)?;

        self.allocate_buffer(return_data)
    }

    fn blueprint_call(
        &mut self,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
        function_ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let (package_address, blueprint_name) =
            Self::parse_blueprint_id(package_address, blueprint_name)?;
        let function_ident =
            String::from_utf8(function_ident).map_err(|_| WasmRuntimeError::InvalidString)?;

        let return_data = self.api.call_function(
            package_address,
            blueprint_name.as_str(),
            &function_ident,
            args,
        )?;

        self.allocate_buffer(return_data)
    }

    fn object_new(
        &mut self,
        blueprint_name: Vec<u8>,
        object_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_name =
            String::from_utf8(blueprint_name).map_err(|_| WasmRuntimeError::InvalidString)?;
        let object_states = scrypto_decode::<IndexMap<u8, FieldValue>>(&object_states)
            .map_err(WasmRuntimeError::InvalidObjectStates)?;

        let component_id = self
            .api
            .new_simple_object(blueprint_name.as_ref(), object_states)?;

        self.allocate_buffer(component_id.to_vec())
    }

    fn address_allocate(
        &mut self,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let (package_address, blueprint_name) =
            Self::parse_blueprint_id(package_address, blueprint_name)?;

        let address_reservation_and_address = self.api.allocate_global_address(BlueprintId {
            package_address,
            blueprint_name,
        })?;
        let encoded = scrypto_encode(&address_reservation_and_address)
            .expect("Failed to encode object address");

        self.allocate_buffer(encoded)
    }

    fn address_get_reservation_address(
        &mut self,
        node_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );

        let address = self.api.get_reservation_address(&node_id)?;
        let address_encoded = scrypto_encode(&address).expect("Failed to encode address");

        self.allocate_buffer(address_encoded)
    }

    fn globalize_object(
        &mut self,
        node_id: Vec<u8>,
        modules: Vec<u8>,
        address_reservation: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let modules = scrypto_decode::<IndexMap<AttachedModuleId, NodeId>>(&modules)
            .map_err(WasmRuntimeError::InvalidModules)?;
        let address_reservation =
            scrypto_decode::<Option<GlobalAddressReservation>>(&address_reservation)
                .map_err(|_| WasmRuntimeError::InvalidGlobalAddressReservation)?;

        let address = self.api.globalize(node_id, modules, address_reservation)?;

        self.allocate_buffer(address.to_vec())
    }

    fn key_value_store_new(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let schema = scrypto_decode::<KeyValueStoreDataSchema>(&schema)
            .map_err(WasmRuntimeError::InvalidKeyValueStoreSchema)?;

        let key_value_store_id = self.api.key_value_store_new(schema)?;

        self.allocate_buffer(key_value_store_id.to_vec())
    }

    fn key_value_store_open_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
        flags: u32,
    ) -> Result<SubstateHandle, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );

        let flags = LockFlags::from_bits(flags).ok_or(WasmRuntimeError::InvalidLockFlags)?;
        let handle = self.api.key_value_store_open_entry(&node_id, &key, flags)?;

        Ok(handle)
    }

    fn key_value_entry_get(
        &mut self,
        handle: u32,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let value = self.api.key_value_entry_get(handle)?;
        self.allocate_buffer(value)
    }

    fn key_value_entry_set(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.key_value_entry_set(handle, data)?;
        Ok(())
    }

    fn key_value_entry_remove(
        &mut self,
        handle: u32,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let value = self.api.key_value_entry_remove(handle)?;
        self.allocate_buffer(value)
    }

    fn key_value_entry_close(&mut self, handle: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.key_value_entry_close(handle)?;
        Ok(())
    }

    fn key_value_store_remove_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let rtn = self.api.key_value_store_remove_entry(&node_id, &key)?;
        self.allocate_buffer(rtn)
    }

    fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: u32,
    ) -> Result<SubstateHandle, InvokeError<WasmRuntimeError>> {
        let flags = LockFlags::from_bits(flags).ok_or(WasmRuntimeError::InvalidLockFlags)?;
        let handle = self.api.actor_open_field(object_handle, field, flags)?;

        Ok(handle)
    }

    fn field_entry_read(
        &mut self,
        handle: SubstateHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let substate = self.api.field_read(handle)?;

        self.allocate_buffer(substate)
    }

    fn field_entry_write(
        &mut self,
        handle: SubstateHandle,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.field_write(handle, data)?;

        Ok(())
    }

    fn field_entry_close(
        &mut self,
        handle: SubstateHandle,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.field_close(handle)?;

        Ok(())
    }

    fn actor_get_node_id(
        &mut self,
        actor_ref_handle: ActorRefHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = self.api.actor_get_node_id(actor_ref_handle)?;

        self.allocate_buffer(node_id.0.to_vec())
    }

    fn actor_get_package_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_id = self.api.actor_get_blueprint_id()?;

        self.allocate_buffer(blueprint_id.package_address.to_vec())
    }

    fn actor_get_blueprint_name(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_id = self.api.actor_get_blueprint_id()?;

        self.allocate_buffer(blueprint_id.blueprint_name.into_bytes())
    }

    #[inline]
    fn consume_wasm_execution_units(
        &mut self,
        n: u32,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        let n = n.saturating_add(self.wasm_execution_units_base);

        if n <= self.wasm_execution_units_buffer {
            self.wasm_execution_units_buffer -= n;
            Ok(())
        } else {
            self.consume_wasm_execution_exceeding_buffer(n)
        }
    }

    fn instance_of(
        &mut self,
        object_id: Vec<u8>,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let object_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(object_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let (package_address, blueprint_name) =
            Self::parse_blueprint_id(package_address, blueprint_name)?;
        let blueprint_id = self.api.get_blueprint_id(&object_id)?;

        if blueprint_id.package_address.eq(&package_address)
            && blueprint_id.blueprint_name.eq(&blueprint_name)
        {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn blueprint_id(
        &mut self,
        object_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let object_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(object_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let blueprint_id = self.api.get_blueprint_id(&object_id)?;

        let mut buf = Vec::new();
        buf.extend(blueprint_id.package_address.as_bytes());
        buf.extend(blueprint_id.blueprint_name.as_bytes());

        self.allocate_buffer(buf)
    }

    fn get_outer_object(
        &mut self,
        node_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let address = self.api.get_outer_object(&node_id)?;

        self.allocate_buffer(address.to_vec())
    }

    fn actor_emit_event(
        &mut self,
        event_name: Vec<u8>,
        event_payload: Vec<u8>,
        event_flags: EventFlags,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.actor_emit_event(
            String::from_utf8(event_name).map_err(|_| WasmRuntimeError::InvalidString)?,
            event_payload,
            event_flags,
        )?;
        Ok(())
    }

    fn sys_log(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.emit_log(
            scrypto_decode::<Level>(&level).map_err(WasmRuntimeError::InvalidLogLevel)?,
            String::from_utf8(message).map_err(|_| WasmRuntimeError::InvalidString)?,
        )?;
        Ok(())
    }

    fn sys_bech32_encode_address(
        &mut self,
        address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let address =
            scrypto_decode::<GlobalAddress>(&address).map_err(WasmRuntimeError::InvalidAddress)?;
        let encoded = self.api.bech32_encode_address(address)?;
        self.allocate_buffer(encoded.into_bytes())
    }

    fn sys_panic(&mut self, message: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api
            .panic(String::from_utf8(message).map_err(|_| WasmRuntimeError::InvalidString)?)?;
        Ok(())
    }

    fn sys_get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let hash = self.api.get_transaction_hash()?;

        self.allocate_buffer(hash.to_vec())
    }

    fn sys_generate_ruid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let ruid = self.api.generate_ruid()?;

        self.allocate_buffer(ruid.to_vec())
    }

    fn costing_get_execution_cost_unit_limit(
        &mut self,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let execution_cost_unit_limit = self.api.execution_cost_unit_limit()?;

        Ok(execution_cost_unit_limit)
    }

    fn costing_get_execution_cost_unit_price(
        &mut self,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let execution_cost_unit_price = self.api.execution_cost_unit_price()?;

        self.allocate_buffer(
            scrypto_encode(&execution_cost_unit_price)
                .expect("Failed to encode execution_cost_unit_price"),
        )
    }

    fn costing_get_finalization_cost_unit_limit(
        &mut self,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let finalization_cost_unit_limit = self.api.finalization_cost_unit_limit()?;

        Ok(finalization_cost_unit_limit)
    }

    fn costing_get_finalization_cost_unit_price(
        &mut self,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let finalization_cost_unit_price = self.api.finalization_cost_unit_price()?;

        self.allocate_buffer(
            scrypto_encode(&finalization_cost_unit_price)
                .expect("Failed to encode finalization_cost_unit_price"),
        )
    }

    fn costing_get_usd_price(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let usd_price = self.api.usd_price()?;
        self.allocate_buffer(
            scrypto_encode(&usd_price).expect("Failed to encode finalization_cost_unit_price"),
        )
    }

    fn costing_get_tip_percentage(&mut self) -> Result<u32, InvokeError<WasmRuntimeError>> {
        Ok(self.api.tip_percentage_truncated()?)
    }

    fn costing_get_fee_balance(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let fee_balance = self.api.fee_balance()?;

        self.allocate_buffer(scrypto_encode(&fee_balance).expect("Failed to encode fee_balance"))
    }

    /// This method is only available to packages uploaded after "Anemone"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=message.len())]
    fn crypto_utils_bls12381_v1_verify(
        &mut self,
        message: Vec<u8>,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let public_key: Bls12381G1PublicKey =
            scrypto_decode(&public_key).map_err(WasmRuntimeError::InvalidBlsPublicKey)?;
        let signature: Bls12381G2Signature =
            scrypto_decode(&signature).map_err(WasmRuntimeError::InvalidBlsSignature)?;

        self.api
            .consume_cost_units(ClientCostingEntry::Bls12381V1Verify {
                size: message.len(),
            })?;

        Ok(verify_bls12381_v1(&message, &public_key, &signature) as u32)
    }

    /// This method is only available to packages uploaded after "Anemone"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=pub_keys_and_msgs.len())]
    fn crypto_utils_bls12381_v1_aggregate_verify(
        &mut self,
        pub_keys_and_msgs: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let signature: Bls12381G2Signature =
            scrypto_decode(&signature).map_err(WasmRuntimeError::InvalidBlsSignature)?;
        let pub_keys_and_msgs: Vec<(Bls12381G1PublicKey, Vec<u8>)> =
            scrypto_decode(&pub_keys_and_msgs)
                .map_err(WasmRuntimeError::InvalidBlsPublicKeyOrMessage)?;

        if pub_keys_and_msgs.is_empty() {
            return Err(InvokeError::SelfError(WasmRuntimeError::InputDataEmpty));
        }

        let sizes: Vec<usize> = pub_keys_and_msgs.iter().map(|(_, msg)| msg.len()).collect();

        self.api
            .consume_cost_units(ClientCostingEntry::Bls12381V1AggregateVerify {
                sizes: sizes.as_slice(),
            })?;

        Ok(aggregate_verify_bls12381_v1(&pub_keys_and_msgs, &signature) as u32)
    }

    /// This method is only available to packages uploaded after "Anemone"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=message.len(), log=public_keys.len())]
    fn crypto_utils_bls12381_v1_fast_aggregate_verify(
        &mut self,
        message: Vec<u8>,
        public_keys: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let public_keys: Vec<Bls12381G1PublicKey> =
            scrypto_decode(&public_keys).map_err(WasmRuntimeError::InvalidBlsPublicKey)?;
        let signature: Bls12381G2Signature =
            scrypto_decode(&signature).map_err(WasmRuntimeError::InvalidBlsSignature)?;

        if public_keys.is_empty() {
            return Err(InvokeError::SelfError(WasmRuntimeError::InputDataEmpty));
        }

        self.api
            .consume_cost_units(ClientCostingEntry::Bls12381V1FastAggregateVerify {
                size: message.len(),
                keys_cnt: public_keys.len(),
            })?;

        if self.scrypto_vm_version == ScryptoVmVersion::crypto_utils_v1() {
            Ok(
                fast_aggregate_verify_bls12381_v1_anemone(&message, &public_keys, &signature)
                    as u32,
            )
        } else {
            Ok(fast_aggregate_verify_bls12381_v1(&message, &public_keys, &signature) as u32)
        }
    }

    /// This method is only available to packages uploaded after "Anemone"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=signatures.len())]
    fn crypto_utils_bls12381_g2_signature_aggregate(
        &mut self,
        signatures: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let signatures: Vec<Bls12381G2Signature> =
            scrypto_decode(&signatures).map_err(WasmRuntimeError::InvalidBlsSignature)?;

        if signatures.is_empty() {
            return Err(InvokeError::SelfError(WasmRuntimeError::InputDataEmpty));
        }

        self.api
            .consume_cost_units(ClientCostingEntry::Bls12381G2SignatureAggregate {
                signatures_cnt: signatures.len(),
            })?;

        let agg_sig = if self.scrypto_vm_version == ScryptoVmVersion::crypto_utils_v1() {
            Bls12381G2Signature::aggregate_anemone(&signatures)
        } else {
            Bls12381G2Signature::aggregate(&signatures, true)
        }
        .map_err(|err| RuntimeError::SystemError(SystemError::BlsError(err.to_string())))?;

        self.allocate_buffer(
            scrypto_encode(&agg_sig).expect("Failed to encode Bls12381G2Signature"),
        )
    }

    /// This method is only available to packages uploaded after "Anemone"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=data.len())]
    fn crypto_utils_keccak256_hash(
        &mut self,
        data: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        self.api
            .consume_cost_units(ClientCostingEntry::Keccak256Hash { size: data.len() })?;

        let hash = keccak256_hash(data);

        self.allocate_buffer(hash.to_vec())
    }

    /// This method is only available to packages uploaded after "Cuttlefish"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=data.len())]
    fn crypto_utils_blake2b_256_hash(
        &mut self,
        data: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        self.api
            .consume_cost_units(ClientCostingEntry::Blake2b256Hash { size: data.len() })?;

        let hash = blake2b_256_hash(data);

        self.allocate_buffer(hash.to_vec())
    }

    /// This method is only available to packages uploaded after "Cuttlefish"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=message.len())]
    fn crypto_utils_ed25519_verify(
        &mut self,
        message: Vec<u8>,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let public_key = Ed25519PublicKey::try_from(public_key.as_ref())
            .map_err(WasmRuntimeError::InvalidEd25519PublicKey)?;
        let signature = Ed25519Signature::try_from(signature.as_ref())
            .map_err(WasmRuntimeError::InvalidEd25519Signature)?;

        self.api
            .consume_cost_units(ClientCostingEntry::Ed25519Verify {
                size: message.len(),
            })?;

        Ok(verify_ed25519(&message, &public_key, &signature) as u32)
    }

    /// This method is only available to packages uploaded after "Cuttlefish"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources(log=message.len())]
    fn crypto_utils_secp256k1_ecdsa_verify(
        &mut self,
        message: Vec<u8>,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let public_key = Secp256k1PublicKey::try_from(public_key.as_ref())
            .map_err(WasmRuntimeError::InvalidSecp256k1PublicKey)?;
        let signature = Secp256k1Signature::try_from(signature.as_ref())
            .map_err(WasmRuntimeError::InvalidSecp256k1Signature)?;
        let hash = Hash::try_from(message.as_slice()).map_err(WasmRuntimeError::InvalidHash)?;

        self.api
            .consume_cost_units(ClientCostingEntry::Secp256k1EcdsaVerify)?;

        Ok(verify_secp256k1(&hash, &public_key, &signature) as u32)
    }

    /// This method is only available to packages uploaded after "Cuttlefish"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources]
    fn crypto_utils_secp256k1_ecdsa_verify_and_key_recover(
        &mut self,
        message: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let hash = Hash::try_from(message.as_slice()).map_err(WasmRuntimeError::InvalidHash)?;
        let signature = Secp256k1Signature::try_from(signature.as_ref())
            .map_err(WasmRuntimeError::InvalidSecp256k1Signature)?;

        self.api
            .consume_cost_units(ClientCostingEntry::Secp256k1EcdsaKeyRecover)?;

        let key = verify_and_recover_secp256k1(&hash, &signature)
            .ok_or(WasmRuntimeError::Secp256k1KeyRecoveryError)?;

        self.allocate_buffer(key.to_vec())
    }

    /// This method is only available to packages uploaded after "Cuttlefish"
    /// protocol update due to checks in [`ScryptoV1WasmValidator::validate`].
    #[trace_resources]
    fn crypto_utils_secp256k1_ecdsa_verify_and_key_recover_uncompressed(
        &mut self,
        message: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let hash = Hash::try_from(message.as_slice()).map_err(WasmRuntimeError::InvalidHash)?;
        let signature = Secp256k1Signature::try_from(signature.as_ref())
            .map_err(WasmRuntimeError::InvalidSecp256k1Signature)?;

        self.api
            .consume_cost_units(ClientCostingEntry::Secp256k1EcdsaKeyRecover)?;

        let key = verify_and_recover_secp256k1_uncompressed(&hash, &signature)
            .ok_or(WasmRuntimeError::Secp256k1KeyRecoveryError)?;

        self.allocate_buffer(key.0.to_vec())
    }
}
