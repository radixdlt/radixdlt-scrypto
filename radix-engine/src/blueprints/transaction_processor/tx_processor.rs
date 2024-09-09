use crate::blueprints::resource::WorktopSubstate;
use crate::blueprints::transaction_processor::{TxnInstruction, Yield};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::node_init::type_info_partition;
use crate::system::type_info::TypeInfoBlueprint;
use crate::system::type_info::TypeInfoSubstate;
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::package::BlueprintVersion;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::*;
use radix_native_sdk::resource::Worktop;
use radix_native_sdk::runtime::LocalAuthZone;
use radix_transactions::data::TransformHandler;
use radix_transactions::model::*;
use radix_transactions::validation::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TransactionProcessorError {
    BucketNotFound(u32),
    ProofNotFound(u32),
    AddressReservationNotFound(u32),
    AddressNotFound(u32),
    BlobNotFound(Hash),
    InvalidCallData(DecodeError),
    InvalidPackageSchema(DecodeError),
    NotPackageAddress(NodeId),
    NotGlobalAddress(NodeId),
    AuthZoneIsEmpty,
    InvocationOutputDecodeError(DecodeError),
    ArgsEncodeError(EncodeError),
    TotalBlobSizeLimitExceeded,
}

impl From<TransactionProcessorError> for RuntimeError {
    fn from(value: TransactionProcessorError) -> Self {
        Self::ApplicationError(ApplicationError::TransactionProcessorError(value))
    }
}

pub struct TxnProcessorThread<I: TxnInstruction + ManifestDecode + ManifestCategorize> {
    instructions: VecDeque<I>,
    worktop: Worktop,
    objects: TxnProcessorObjects,
    pub instruction_index: usize,
    pub outputs: Vec<InstructionOutput>,
}

impl<I: TxnInstruction + ManifestDecode + ManifestCategorize> TxnProcessorThread<I> {
    pub fn init<Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<L>, L: Default>(
        manifest_encoded_instructions: Rc<Vec<u8>>,
        global_address_reservations: Vec<GlobalAddressReservation>,
        blobs: Rc<IndexMap<Hash, Vec<u8>>>,
        max_total_size_of_blobs: usize,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        // Create a worktop
        let worktop_node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            worktop_node_id,
            btreemap!(
                MAIN_BASE_PARTITION => btreemap!(
                    WorktopField::Worktop.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(WorktopSubstate::new()))
                ),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint_info: BlueprintInfo {
                            blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT),
                            blueprint_version: BlueprintVersion::default(),
                            generic_substitutions: Vec::new(),
                            outer_obj_info: OuterObjectInfo::default(),
                            features: indexset!(),
                        },
                        object_type: ObjectType::Owned,
                    })
                )
            ),
        )?;
        api.kernel_pin_node(worktop_node_id)?;

        let worktop = Worktop(Own(worktop_node_id));
        let instructions =
            manifest_decode::<Vec<I>>(&manifest_encoded_instructions).map_err(|e| {
                // This error should never occur if being called from root since this is constructed
                // by the transaction executor. This error is more to protect against application
                // space calling this function if/when possible
                RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
            })?;
        let objects =
            TxnProcessorObjects::new(blobs, global_address_reservations, max_total_size_of_blobs);
        let outputs = Vec::new();

        Ok(Self {
            instructions: instructions.into_iter().collect(),
            instruction_index: 0usize,
            worktop,
            objects,
            outputs,
        })
    }

    pub fn resume<Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<L>, L: Default>(
        &mut self,
        api: &mut Y,
    ) -> Result<Option<Yield>, RuntimeError> {
        while let Some(instruction) = self.instructions.pop_front() {
            api.update_instruction_index(self.instruction_index)?;
            let (output, yield_instruction) =
                instruction.execute(&mut self.worktop, &mut self.objects, api)?;
            self.outputs.push(output);
            self.instruction_index += 1;
            if yield_instruction.is_some() {
                return Ok(yield_instruction);
            }
        }

        self.worktop.drop(api)?;

        Ok(None)
    }
}

pub struct TxnProcessorObjects {
    bucket_mapping: NonIterMap<ManifestBucket, NodeId>,
    pub proof_mapping: IndexMap<ManifestProof, NodeId>,
    address_reservation_mapping: NonIterMap<ManifestAddressReservation, NodeId>,
    address_mapping: NonIterMap<u32, NodeId>,
    id_allocator: ManifestIdAllocator,
    blobs_by_hash: Rc<IndexMap<Hash, Vec<u8>>>,
    max_total_size_of_blobs: usize,
}

impl TxnProcessorObjects {
    fn new(
        blobs_by_hash: Rc<IndexMap<Hash, Vec<u8>>>,
        global_address_reservations: Vec<GlobalAddressReservation>,
        max_total_size_of_blobs: usize,
    ) -> Self {
        let mut processor = Self {
            blobs_by_hash,
            proof_mapping: index_map_new(),
            bucket_mapping: NonIterMap::new(),
            address_reservation_mapping: NonIterMap::new(),
            address_mapping: NonIterMap::new(),
            id_allocator: ManifestIdAllocator::new(),
            max_total_size_of_blobs,
        };

        for address_reservation in global_address_reservations {
            processor
                .create_manifest_address_reservation(address_reservation)
                .unwrap();
        }
        processor
    }

    pub fn get_bucket(&mut self, bucket_id: &ManifestBucket) -> Result<Bucket, RuntimeError> {
        let real_id =
            self.bucket_mapping
                .get(bucket_id)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::BucketNotFound(bucket_id.0),
                    ),
                ))?;
        Ok(Bucket(Own(real_id)))
    }

    pub fn take_bucket(&mut self, bucket_id: &ManifestBucket) -> Result<Bucket, RuntimeError> {
        let real_id =
            self.bucket_mapping
                .remove(bucket_id)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::BucketNotFound(bucket_id.0),
                    ),
                ))?;
        Ok(Bucket(Own(real_id)))
    }

    pub fn get_blob(&mut self, blob_ref: &ManifestBlobRef) -> Result<&[u8], RuntimeError> {
        let hash = Hash(blob_ref.0);
        self.blobs_by_hash
            .get(&hash)
            .map(|x| x.as_ref())
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::BlobNotFound(hash),
                ),
            ))
    }

    pub fn get_proof(&mut self, proof_id: &ManifestProof) -> Result<Proof, RuntimeError> {
        let real_id =
            self.proof_mapping
                .get(proof_id)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ProofNotFound(proof_id.0),
                    ),
                ))?;
        Ok(Proof(Own(real_id)))
    }

    pub fn get_address(&mut self, address_id: &u32) -> Result<NodeId, RuntimeError> {
        let real_id =
            self.address_mapping
                .get(address_id)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::AddressNotFound(*address_id),
                    ),
                ))?;
        Ok(real_id)
    }

    pub fn take_proof(&mut self, proof_id: &ManifestProof) -> Result<Proof, RuntimeError> {
        let real_id =
            self.proof_mapping
                .swap_remove(proof_id)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ProofNotFound(proof_id.0),
                    ),
                ))?;
        Ok(Proof(Own(real_id)))
    }

    pub fn take_address_reservation(
        &mut self,
        address_reservation_id: &ManifestAddressReservation,
    ) -> Result<GlobalAddressReservation, RuntimeError> {
        let real_id = self
            .address_reservation_mapping
            .remove(address_reservation_id)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::AddressReservationNotFound(address_reservation_id.0),
                ),
            ))?;
        Ok(GlobalAddressReservation(Own(real_id)))
    }

    pub fn create_manifest_bucket(&mut self, bucket: Bucket) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_bucket_id();
        self.bucket_mapping.insert(new_id.clone(), bucket.0.into());
        Ok(())
    }

    pub fn create_manifest_proof(&mut self, proof: Proof) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_proof_id();
        self.proof_mapping.insert(new_id.clone(), proof.0.into());
        Ok(())
    }

    pub fn create_manifest_address_reservation(
        &mut self,
        address_reservation: GlobalAddressReservation,
    ) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_address_reservation_id();
        self.address_reservation_mapping
            .insert(new_id, address_reservation.0.into());
        Ok(())
    }

    pub fn create_manifest_address(&mut self, address: GlobalAddress) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_address_id();
        self.address_mapping.insert(new_id, address.into());
        Ok(())
    }

    pub fn resolve_package_address(
        &mut self,
        address: DynamicPackageAddress,
    ) -> Result<PackageAddress, RuntimeError> {
        match address {
            DynamicPackageAddress::Static(address) => Ok(address),
            DynamicPackageAddress::Named(name) => {
                let node_id = self.get_address(&name)?;
                PackageAddress::try_from(node_id.0).map_err(|_| {
                    RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::NotPackageAddress(node_id),
                    ))
                })
            }
        }
    }

    pub fn resolve_global_address(
        &mut self,
        address: DynamicGlobalAddress,
    ) -> Result<GlobalAddress, RuntimeError> {
        match address {
            DynamicGlobalAddress::Static(address) => Ok(address),
            DynamicGlobalAddress::Named(name) => {
                let node_id = self.get_address(&name)?;
                GlobalAddress::try_from(node_id.0).map_err(|_| {
                    RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::NotGlobalAddress(node_id),
                    ))
                })
            }
        }
    }

    pub fn handle_call_return_data<
        Y: SystemApi<RuntimeError> + KernelSubstateApi<L>,
        L: Default,
    >(
        &mut self,
        value: &IndexedScryptoValue,
        worktop: &Worktop,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // Auto move into worktop & auth_zone
        for node_id in value.owned_nodes() {
            let info = TypeInfoBlueprint::get_type(node_id, api)?;
            match info {
                TypeInfoSubstate::Object(info) => match (
                    info.blueprint_info.blueprint_id.package_address,
                    info.blueprint_info.blueprint_id.blueprint_name.as_str(),
                ) {
                    (RESOURCE_PACKAGE, FUNGIBLE_BUCKET_BLUEPRINT)
                    | (RESOURCE_PACKAGE, NON_FUNGIBLE_BUCKET_BLUEPRINT) => {
                        let bucket = Bucket(Own(node_id.clone()));
                        worktop.put(bucket, api)?;
                    }
                    (RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT)
                    | (RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT) => {
                        let proof = Proof(Own(node_id.clone()));
                        LocalAuthZone::push(proof, api)?;
                    }
                    _ => {
                        // No-op, but can be extended
                    }
                },
                TypeInfoSubstate::KeyValueStore(_)
                | TypeInfoSubstate::GlobalAddressReservation(_)
                | TypeInfoSubstate::GlobalAddressPhantom(_) => {
                    // No-op, but can be extended
                }
            }
        }

        Ok(())
    }
}

pub struct TxnProcessorObjectsWithApi<'a, 'p, 'w, Y: SystemApi<RuntimeError>> {
    pub(crate) worktop: &'w mut Worktop,
    pub(crate) objects: &'p mut TxnProcessorObjects,
    pub(crate) api: &'a mut Y,
    pub(crate) current_total_size_of_blobs: usize,
}

impl<'a, 'p, 'w, Y: SystemApi<RuntimeError>> TransformHandler<RuntimeError>
    for TxnProcessorObjectsWithApi<'a, 'p, 'w, Y>
{
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, RuntimeError> {
        self.objects.take_bucket(&b).map(|x| x.0)
    }

    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, RuntimeError> {
        self.objects.take_proof(&p).map(|x| x.0)
    }

    fn replace_address_reservation(
        &mut self,
        r: ManifestAddressReservation,
    ) -> Result<Own, RuntimeError> {
        self.objects.take_address_reservation(&r).map(|x| x.0)
    }

    fn replace_named_address(&mut self, a: u32) -> Result<Reference, RuntimeError> {
        self.objects.get_address(&a).map(|x| Reference(x))
    }

    fn replace_expression(&mut self, e: ManifestExpression) -> Result<Vec<Own>, RuntimeError> {
        match e {
            ManifestExpression::EntireWorktop => {
                let buckets = self.worktop.drain(self.api)?;
                Ok(buckets.into_iter().map(|b| b.0).collect())
            }
            ManifestExpression::EntireAuthZone => {
                let proofs = LocalAuthZone::drain(self.api)?;
                Ok(proofs.into_iter().map(|p| p.0).collect())
            }
        }
    }

    fn replace_blob(&mut self, b: ManifestBlobRef) -> Result<Vec<u8>, RuntimeError> {
        let max_total_size_of_blobs = self.objects.max_total_size_of_blobs;
        let blob = self.objects.get_blob(&b)?;

        if let Some(new_total) = self.current_total_size_of_blobs.checked_add(blob.len()) {
            if new_total <= max_total_size_of_blobs {
                self.current_total_size_of_blobs = new_total;
                return Ok(blob.to_vec());
            }
        }

        Err(RuntimeError::ApplicationError(
            ApplicationError::TransactionProcessorError(
                TransactionProcessorError::TotalBlobSizeLimitExceeded,
            ),
        ))
    }
}
