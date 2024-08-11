use crate::blueprints::resource::WorktopSubstate;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::node_init::type_info_partition;
use crate::system::type_info::TypeInfoBlueprint;
use crate::system::type_info::TypeInfoSubstate;
use radix_engine_interface::api::{AttachedModuleId, SystemApi};
use radix_engine_interface::blueprints::package::BlueprintVersion;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::*;
use radix_native_sdk::resource::NativeFungibleBucket;
use radix_native_sdk::resource::NativeNonFungibleBucket;
use radix_native_sdk::resource::{NativeBucket, NativeProof, Worktop};
use radix_native_sdk::runtime::LocalAuthZone;
use radix_transactions::data::transform;
use radix_transactions::data::TransformHandler;
use radix_transactions::model::*;
use radix_transactions::validation::*;
use sbor::rust::prelude::*;

#[cfg(not(feature = "coverage"))]
pub const MAX_TOTAL_BLOB_SIZE_PER_INVOCATION: usize = 1024 * 1024;
#[cfg(feature = "coverage")]
pub const MAX_TOTAL_BLOB_SIZE_PER_INVOCATION: usize = 64 * 1024 * 1024;

/// The minor version of the TransactionProcessor V1 package
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum TransactionProcessorV1MinorVersion {
    Zero,
    One,
}

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionProcessorRunInput {
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<GlobalAddressReservation>,
    pub references: Vec<Reference>, // Required so that the kernel passes the references to the processor frame
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

// This needs to match the above, but is easily encodable to avoid cloning from the transaction payload to encode
#[derive(Debug, Eq, PartialEq, ScryptoEncode)]
pub struct TransactionProcessorRunInputEfficientEncodable {
    pub manifest_encoded_instructions: Rc<Vec<u8>>,
    pub global_address_reservations: Vec<GlobalAddressReservation>,
    pub references: IndexSet<Reference>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
}

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

fn handle_invocation<'a, 'p, 'w, Y: SystemApi<RuntimeError> + KernelSubstateApi<L>, L: Default>(
    api: &'a mut Y,
    processor: &'p mut TransactionProcessorMapping,
    worktop: &'w mut Worktop,
    args: ManifestValue,
    invocation_handler: impl FnOnce(&mut Y, ScryptoValue) -> Result<Vec<u8>, RuntimeError>,
    version: TransactionProcessorV1MinorVersion,
) -> Result<InstructionOutput, RuntimeError> {
    let scrypto_value = {
        let mut processor_with_api = TransactionProcessorWithApi {
            worktop,
            processor,
            api,
            current_total_size_of_blobs: 0,
            max_total_size_of_blobs: match version {
                TransactionProcessorV1MinorVersion::Zero => usize::MAX,
                TransactionProcessorV1MinorVersion::One => MAX_TOTAL_BLOB_SIZE_PER_INVOCATION,
            },
        };
        transform(args, &mut processor_with_api)?
    };

    let rtn = invocation_handler(api, scrypto_value)?;

    let result = IndexedScryptoValue::from_vec(rtn)
        .map_err(|error| TransactionProcessorError::InvocationOutputDecodeError(error))?;
    processor.handle_call_return_data(&result, &worktop, api)?;
    Ok(InstructionOutput::CallReturn(result.into()))
}

pub struct TransactionProcessorBlueprint;

impl TransactionProcessorBlueprint {
    pub(crate) fn run<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<L>,
        L: Default,
    >(
        manifest_encoded_instructions: Vec<u8>,
        global_address_reservations: Vec<GlobalAddressReservation>,
        _references: Vec<Reference>, // Required so that the kernel passes the references to the processor frame
        blobs: IndexMap<Hash, Vec<u8>>,
        version: TransactionProcessorV1MinorVersion,
        api: &mut Y,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        let processor = TransactionProcessor::init(
            manifest_encoded_instructions,
            global_address_reservations,
            blobs,
            version,
            api,
        )?;
        processor.execute(api)
    }
}

struct TransactionProcessor {
    version: TransactionProcessorV1MinorVersion,
    worktop: Worktop,
    instructions: Vec<InstructionV1>,
    processor: TransactionProcessorMapping,
    outputs: Vec<InstructionOutput>,
}

impl TransactionProcessor {
    fn init<Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<L>, L: Default>(
        manifest_encoded_instructions: Vec<u8>,
        global_address_reservations: Vec<GlobalAddressReservation>,
        blobs: IndexMap<Hash, Vec<u8>>,
        version: TransactionProcessorV1MinorVersion,
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

        let mut worktop = Worktop(Own(worktop_node_id));
        let instructions = manifest_decode::<Vec<InstructionV1>>(&manifest_encoded_instructions)
            .map_err(|e| {
                // This error should never occur if being called from root since this is constructed
                // by the transaction executor. This error is more to protect against application
                // space calling this function if/when possible
                RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
            })?;
        let mut processor = TransactionProcessorMapping::new(blobs, global_address_reservations);
        let mut outputs = Vec::new();

        Ok(Self {
            version,
            worktop,
            instructions,
            processor,
            outputs,
        })
    }

    fn execute<Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<L>, L: Default>(
        mut self,
        api: &mut Y,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        for (index, inst) in self.instructions.into_iter().enumerate() {
            api.update_instruction_index(index)?;

            let result = match inst {
                InstructionV1::TakeAllFromWorktop { resource_address } => {
                    let bucket = self.worktop.take_all(resource_address, api)?;
                    self.processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                InstructionV1::TakeFromWorktop {
                    amount,
                    resource_address,
                } => {
                    let bucket = self.worktop.take(resource_address, amount, api)?;
                    self.processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                InstructionV1::TakeNonFungiblesFromWorktop {
                    ids,
                    resource_address,
                } => {
                    let bucket = self.worktop.take_non_fungibles(
                        resource_address,
                        ids.into_iter().collect(),
                        api,
                    )?;
                    self.processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                InstructionV1::ReturnToWorktop { bucket_id } => {
                    let bucket = self.processor.take_bucket(&bucket_id)?;
                    self.worktop.put(bucket, api)?;
                    InstructionOutput::None
                }
                InstructionV1::AssertWorktopContainsAny { resource_address } => {
                    self.worktop.assert_contains(resource_address, api)?;
                    InstructionOutput::None
                }
                InstructionV1::AssertWorktopContains {
                    amount,
                    resource_address,
                } => {
                    self.worktop
                        .assert_contains_amount(resource_address, amount, api)?;
                    InstructionOutput::None
                }
                InstructionV1::AssertWorktopContainsNonFungibles {
                    ids,
                    resource_address,
                } => {
                    self.worktop.assert_contains_non_fungibles(
                        resource_address,
                        ids.into_iter().collect(),
                        api,
                    )?;
                    InstructionOutput::None
                }
                InstructionV1::PopFromAuthZone {} => {
                    let proof = LocalAuthZone::pop(api)?.ok_or(RuntimeError::ApplicationError(
                        ApplicationError::TransactionProcessorError(
                            TransactionProcessorError::AuthZoneIsEmpty,
                        ),
                    ))?;
                    self.processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::PushToAuthZone { proof_id } => {
                    let proof = self.processor.take_proof(&proof_id)?;
                    LocalAuthZone::push(proof, api)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromAuthZoneOfAmount {
                    amount,
                    resource_address,
                } => {
                    let proof =
                        LocalAuthZone::create_proof_of_amount(amount, resource_address, api)?;
                    self.processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
                    ids,
                    resource_address,
                } => {
                    let proof = LocalAuthZone::create_proof_of_non_fungibles(
                        &ids.into_iter().collect(),
                        resource_address,
                        api,
                    )?;
                    self.processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromAuthZoneOfAll { resource_address } => {
                    let proof = LocalAuthZone::create_proof_of_all(resource_address, api)?;
                    self.processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromBucketOfAmount { bucket_id, amount } => {
                    let bucket = self.processor.get_bucket(&bucket_id)?;
                    let proof = bucket.create_proof_of_amount(amount, api)?;
                    self.processor.create_manifest_proof(proof.into())?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromBucketOfNonFungibles { bucket_id, ids } => {
                    let bucket = self.processor.get_bucket(&bucket_id)?;
                    let proof =
                        bucket.create_proof_of_non_fungibles(ids.into_iter().collect(), api)?;
                    self.processor.create_manifest_proof(proof.into())?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromBucketOfAll { bucket_id } => {
                    let bucket = self.processor.get_bucket(&bucket_id)?;
                    let proof = bucket.create_proof_of_all(api)?;
                    self.processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::DropAuthZoneProofs => {
                    LocalAuthZone::drop_proofs(api)?;
                    InstructionOutput::None
                }
                InstructionV1::DropAuthZoneRegularProofs => {
                    LocalAuthZone::drop_regular_proofs(api)?;
                    InstructionOutput::None
                }
                InstructionV1::DropAuthZoneSignatureProofs => {
                    LocalAuthZone::drop_signature_proofs(api)?;
                    InstructionOutput::None
                }
                InstructionV1::BurnResource { bucket_id } => {
                    let bucket = self.processor.take_bucket(&bucket_id)?;
                    let rtn = bucket.burn(api)?;

                    let result = IndexedScryptoValue::from_typed(&rtn);
                    self.processor
                        .handle_call_return_data(&result, &self.worktop, api)?;
                    InstructionOutput::CallReturn(result.into())
                }
                InstructionV1::CloneProof { proof_id } => {
                    let proof = self.processor.get_proof(&proof_id)?;
                    let proof = proof.clone(api)?;
                    self.processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::DropProof { proof_id } => {
                    let proof = self.processor.take_proof(&proof_id)?;
                    proof.drop(api)?;
                    InstructionOutput::None
                }
                InstructionV1::CallFunction {
                    package_address,
                    blueprint_name,
                    function_name,
                    args,
                } => {
                    let package_address =
                        self.processor.resolve_package_address(package_address)?;
                    handle_invocation(
                        api,
                        &mut self.processor,
                        &mut self.worktop,
                        args,
                        |api, args| {
                            api.call_function(
                                package_address,
                                &blueprint_name,
                                &function_name,
                                scrypto_encode(&args)
                                    .map_err(TransactionProcessorError::ArgsEncodeError)?,
                            )
                        },
                        self.version,
                    )?
                }
                InstructionV1::CallMethod {
                    address,
                    method_name,
                    args,
                } => {
                    let address = self.processor.resolve_global_address(address)?;
                    handle_invocation(
                        api,
                        &mut self.processor,
                        &mut self.worktop,
                        args,
                        |api, args| {
                            api.call_method(
                                address.as_node_id(),
                                &method_name,
                                scrypto_encode(&args)
                                    .map_err(TransactionProcessorError::ArgsEncodeError)?,
                            )
                        },
                        self.version,
                    )?
                }
                InstructionV1::CallRoyaltyMethod {
                    address,
                    method_name,
                    args,
                } => {
                    let address = self.processor.resolve_global_address(address)?;
                    handle_invocation(
                        api,
                        &mut self.processor,
                        &mut self.worktop,
                        args,
                        |api, args| {
                            api.call_module_method(
                                address.as_node_id(),
                                AttachedModuleId::Royalty,
                                &method_name,
                                scrypto_encode(&args)
                                    .map_err(TransactionProcessorError::ArgsEncodeError)?,
                            )
                        },
                        self.version,
                    )?
                }
                InstructionV1::CallMetadataMethod {
                    address,
                    method_name,
                    args,
                } => {
                    let address = self.processor.resolve_global_address(address)?;
                    handle_invocation(
                        api,
                        &mut self.processor,
                        &mut self.worktop,
                        args,
                        |api, args| {
                            api.call_module_method(
                                address.as_node_id(),
                                AttachedModuleId::Metadata,
                                &method_name,
                                scrypto_encode(&args)
                                    .map_err(TransactionProcessorError::ArgsEncodeError)?,
                            )
                        },
                        self.version,
                    )?
                }
                InstructionV1::CallRoleAssignmentMethod {
                    address,
                    method_name,
                    args,
                } => {
                    let address = self.processor.resolve_global_address(address)?;
                    handle_invocation(
                        api,
                        &mut self.processor,
                        &mut self.worktop,
                        args,
                        |api, args| {
                            api.call_module_method(
                                address.as_node_id(),
                                AttachedModuleId::RoleAssignment,
                                &method_name,
                                scrypto_encode(&args)
                                    .map_err(TransactionProcessorError::ArgsEncodeError)?,
                            )
                        },
                        self.version,
                    )?
                }
                InstructionV1::CallDirectVaultMethod {
                    address,
                    method_name,
                    args,
                } => handle_invocation(
                    api,
                    &mut self.processor,
                    &mut self.worktop,
                    args,
                    |api, args| {
                        api.call_direct_access_method(
                            address.as_node_id(),
                            &method_name,
                            scrypto_encode(&args)
                                .map_err(TransactionProcessorError::ArgsEncodeError)?,
                        )
                    },
                    self.version,
                )?,
                InstructionV1::DropNamedProofs => {
                    for (_, real_id) in self.processor.proof_mapping.drain(..) {
                        let proof = Proof(Own(real_id));
                        proof.drop(api).map(|_| IndexedScryptoValue::unit())?;
                    }
                    InstructionOutput::None
                }
                InstructionV1::DropAllProofs => {
                    for (_, real_id) in self.processor.proof_mapping.drain(..) {
                        let proof = Proof(Own(real_id));
                        proof.drop(api).map(|_| IndexedScryptoValue::unit())?;
                    }
                    LocalAuthZone::drop_proofs(api)?;
                    InstructionOutput::None
                }
                InstructionV1::AllocateGlobalAddress {
                    package_address,
                    blueprint_name,
                } => {
                    let (address_reservation, address) = api.allocate_global_address(
                        BlueprintId::new(&package_address, blueprint_name),
                    )?;
                    self.processor
                        .create_manifest_address_reservation(address_reservation)?;
                    self.processor.create_manifest_address(address)?;

                    InstructionOutput::None
                }
            };
            self.outputs.push(result);
        }

        self.worktop.drop(api)?;

        Ok(self.outputs)
    }
}

struct TransactionProcessorMapping {
    bucket_mapping: NonIterMap<ManifestBucket, NodeId>,
    proof_mapping: IndexMap<ManifestProof, NodeId>,
    address_reservation_mapping: NonIterMap<ManifestAddressReservation, NodeId>,
    address_mapping: NonIterMap<u32, NodeId>,
    id_allocator: ManifestIdAllocator,
    blobs_by_hash: IndexMap<Hash, Vec<u8>>,
}

impl TransactionProcessorMapping {
    fn new(
        blobs_by_hash: IndexMap<Hash, Vec<u8>>,
        global_address_reservations: Vec<GlobalAddressReservation>,
    ) -> Self {
        let mut processor = Self {
            blobs_by_hash,
            proof_mapping: index_map_new(),
            bucket_mapping: NonIterMap::new(),
            address_reservation_mapping: NonIterMap::new(),
            address_mapping: NonIterMap::new(),
            id_allocator: ManifestIdAllocator::new(),
        };

        for address_reservation in global_address_reservations {
            processor
                .create_manifest_address_reservation(address_reservation)
                .unwrap();
        }
        processor
    }

    fn get_bucket(&mut self, bucket_id: &ManifestBucket) -> Result<Bucket, RuntimeError> {
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

    fn take_bucket(&mut self, bucket_id: &ManifestBucket) -> Result<Bucket, RuntimeError> {
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

    fn get_blob(&mut self, blob_ref: &ManifestBlobRef) -> Result<&[u8], RuntimeError> {
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

    fn get_proof(&mut self, proof_id: &ManifestProof) -> Result<Proof, RuntimeError> {
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

    fn get_address(&mut self, address_id: &u32) -> Result<NodeId, RuntimeError> {
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

    fn take_proof(&mut self, proof_id: &ManifestProof) -> Result<Proof, RuntimeError> {
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

    fn take_address_reservation(
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

    fn create_manifest_bucket(&mut self, bucket: Bucket) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_bucket_id();
        self.bucket_mapping.insert(new_id.clone(), bucket.0.into());
        Ok(())
    }

    fn create_manifest_proof(&mut self, proof: Proof) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_proof_id();
        self.proof_mapping.insert(new_id.clone(), proof.0.into());
        Ok(())
    }

    fn create_manifest_address_reservation(
        &mut self,
        address_reservation: GlobalAddressReservation,
    ) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_address_reservation_id();
        self.address_reservation_mapping
            .insert(new_id, address_reservation.0.into());
        Ok(())
    }

    fn create_manifest_address(&mut self, address: GlobalAddress) -> Result<(), RuntimeError> {
        let new_id = self.id_allocator.new_address_id();
        self.address_mapping.insert(new_id, address.into());
        Ok(())
    }

    fn resolve_package_address(
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

    fn resolve_global_address(
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

    fn handle_call_return_data<Y: SystemApi<RuntimeError> + KernelSubstateApi<L>, L: Default>(
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

struct TransactionProcessorWithApi<'a, 'p, 'w, Y: SystemApi<RuntimeError>> {
    worktop: &'w mut Worktop,
    processor: &'p mut TransactionProcessorMapping,
    api: &'a mut Y,
    current_total_size_of_blobs: usize,
    max_total_size_of_blobs: usize,
}

impl<'a, 'p, 'w, Y: SystemApi<RuntimeError>> TransformHandler<RuntimeError>
    for TransactionProcessorWithApi<'a, 'p, 'w, Y>
{
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, RuntimeError> {
        self.processor.take_bucket(&b).map(|x| x.0)
    }

    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, RuntimeError> {
        self.processor.take_proof(&p).map(|x| x.0)
    }

    fn replace_address_reservation(
        &mut self,
        r: ManifestAddressReservation,
    ) -> Result<Own, RuntimeError> {
        self.processor.take_address_reservation(&r).map(|x| x.0)
    }

    fn replace_named_address(&mut self, a: u32) -> Result<Reference, RuntimeError> {
        self.processor.get_address(&a).map(|x| Reference(x))
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
        let blob = self.processor.get_blob(&b)?;

        if let Some(new_total) = self.current_total_size_of_blobs.checked_add(blob.len()) {
            if new_total <= self.max_total_size_of_blobs {
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
