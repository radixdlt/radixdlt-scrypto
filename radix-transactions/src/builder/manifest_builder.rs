use crate::internal_prelude::*;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::*;
use radix_engine_interface::object_modules::role_assignment::*;
use radix_engine_interface::object_modules::royalty::*;
use radix_engine_interface::object_modules::ModuleConfig;

/// A manifest builder for use in tests.
///
/// Note - if you break invariants of the manifest builder (e.g. resolve a bucket
/// before it's been created, or pass an invalid parameter), this builder will panic.
/// As such, it's only designed for use in test code, or where the inputs are trusted.
///
/// Simple use case:
/// ```
/// # use radix_transactions::prelude::*;
/// # use radix_engine_interface::prelude::*;
/// # use radix_common::prelude::*;
/// # let from_account_address = ComponentAddress::preallocated_account_from_public_key(
/// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
/// # );
/// # let to_account_address = ComponentAddress::preallocated_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let manifest = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .withdraw_from_account(from_account_address, XRD, dec!(1))
///     .take_from_worktop(XRD, dec!(1), "xrd")
///     .try_deposit_or_abort(to_account_address, None, "xrd")
///     .build();
/// ```
///
/// Intermediate use case, where we need to pass a bucket into a component:
/// ```
/// # use radix_transactions::prelude::*;
/// # use radix_engine_interface::prelude::*;
/// # use radix_common::prelude::*;
/// # let package_address = RESOURCE_PACKAGE; // Just some address to get it to compile
/// # let from_account_address = ComponentAddress::preallocated_account_from_public_key(
/// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
/// # );
/// # let to_account_address = ComponentAddress::preallocated_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let manifest = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .withdraw_from_account(from_account_address, XRD, dec!(1))
///     .take_from_worktop(XRD, dec!(1), "xrd")
///     .call_function_with_name_lookup(
///         package_address,
///         "SomeBlueprint",
///         "some_function",
///         |lookup| (
///             lookup.bucket("xrd"),
///         ),
///     )
///     .build();
/// ```
///
/// Advanced use case, where we need to generate a collision-free bucket name:
/// ```
/// # use radix_transactions::prelude::*;
/// # use radix_engine_interface::prelude::*;
/// # use radix_common::prelude::*;
/// # let to_account_address = ComponentAddress::preallocated_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let mut builder = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .get_free_xrd_from_faucet();
/// for _ in 0..32 {
///     // The generate_bucket_name method generates a new bucket name starting with
///     // "transfer" that doesn't collide with any previously used bucket names
///     let bucket_name = builder.generate_bucket_name("transfer");
///     builder = builder
///         .take_from_worktop(XRD, "0.001", &bucket_name)
///         .try_deposit_or_abort(to_account_address, None, bucket_name);
/// }
/// let manifest = builder.build();
/// ```
pub struct ManifestBuilder<M: BuildableManifest = TransactionManifestV1> {
    registrar: ManifestNameRegistrar,
    manifest: M,
}

pub struct NewSymbols {
    pub new_bucket: Option<ManifestBucket>,
    pub new_proof: Option<ManifestProof>,
    pub new_address_reservation: Option<ManifestAddressReservation>,
    pub new_address_id: Option<ManifestNamedAddress>,
}

pub type ManifestV1Builder = ManifestBuilder<TransactionManifestV1>;
pub type ManifestV2Builder = ManifestBuilder<TransactionManifestV2>;
pub type SystemV1ManifestBuilder = ManifestBuilder<SystemTransactionManifestV1>;

impl ManifestV1Builder {
    /// To create a Manifest Builder of a specific version, you may
    /// wish to use a specific new method such as `new_v1()`, `new_v2()`
    /// or `new_system_v1()`.
    ///
    /// For backwards compatibility, we had to keep
    /// `ManifestBuilder::new()` creating a [`ManifestV1Builder`].
    pub fn new() -> Self {
        Self::new_typed()
    }

    /// This exists so that you can call `ManifestBuilder::new_v1()`.
    /// It is equivalent to:
    /// * `ManifestBuilder::<TransactionManifestV1>::new_typed()`
    /// * `ManifestV1Builder::new_typed()`
    pub fn new_v1() -> Self {
        Self::new_typed()
    }
}

impl ManifestV2Builder {
    /// This exists so that you can call `ManifestBuilder::new_v2()`.
    /// It is equivalent to:
    /// * `ManifestBuilder::<TransactionManifestV2>::new_typed()`
    /// * `ManifestV2Builder::new_typed()`
    ///
    /// For backwards compatibility, we had to keep
    /// `ManifestBuilder::new()` creating a [`ManifestV1Builder`].
    pub fn new_v2() -> Self {
        Self::new_typed()
    }
}

impl SystemV1ManifestBuilder {
    /// This exists so that you can call `ManifestBuilder::new_v1()`.
    /// It is equivalent to:
    /// * `ManifestBuilder::<SystemTransactionManifestV1>::new_typed()`
    /// * `SystemV1ManifestBuilder::new_typed()`
    pub fn new_system_v1() -> Self {
        Self::new_typed()
    }

    pub fn add_address_preallocation(
        &mut self,
        fixed_address: impl Into<GlobalAddress>,
        package_address: impl Into<PackageAddress>,
        blueprint_name: impl Into<String>,
    ) -> ManifestAddressReservation {
        let existing_preallocation_count = self.manifest.preallocated_addresses.len();
        let name = format!("preallocation_{existing_preallocation_count}");
        let reservation = self.registrar.new_address_reservation(&name);
        self.preallocate_address_internal(
            reservation,
            fixed_address,
            package_address,
            blueprint_name,
        );
        self.name_lookup().address_reservation(name)
    }

    pub fn preallocate_address(
        mut self,
        reservation: impl NewManifestAddressReservation,
        fixed_address: impl Into<GlobalAddress>,
        package_address: impl Into<PackageAddress>,
        blueprint_name: impl Into<String>,
    ) -> Self {
        self.preallocate_address_internal(
            reservation,
            fixed_address,
            package_address,
            blueprint_name,
        );
        self
    }

    pub fn preallocate_address_internal(
        &mut self,
        reservation: impl NewManifestAddressReservation,
        fixed_address: impl Into<GlobalAddress>,
        package_address: impl Into<PackageAddress>,
        blueprint_name: impl Into<String>,
    ) {
        if self
            .registrar
            .object_names()
            .address_reservation_names
            .len()
            > self.manifest.preallocated_addresses.len()
        {
            panic!("You cannot call preallocate_address after you've allocated any addresses in the manifest");
        }
        self.manifest
            .preallocated_addresses
            .push(PreAllocatedAddress {
                blueprint_id: BlueprintId {
                    package_address: package_address.into(),
                    blueprint_name: blueprint_name.into(),
                },
                address: fixed_address.into(),
            });
        reservation.register(&self.registrar);
    }
}

impl<M: BuildableManifest> ManifestBuilder<M> {
    pub fn new_typed() -> Self {
        Self {
            registrar: ManifestNameRegistrar::new(),
            manifest: M::default(),
        }
    }

    pub fn name_lookup(&self) -> ManifestNameLookup {
        self.registrar.name_lookup()
    }

    pub fn then(self, next: impl FnOnce(Self) -> Self) -> Self {
        next(self)
    }

    pub fn with_name_lookup(self, next: impl FnOnce(Self, ManifestNameLookup) -> Self) -> Self {
        let lookup = self.name_lookup();
        next(self, lookup)
    }

    pub fn with_bucket(
        self,
        bucket: impl ExistingManifestBucket,
        next: impl FnOnce(Self, ManifestBucket) -> Self,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        next(self, bucket)
    }

    pub fn bucket(&self, name: impl AsRef<str>) -> ManifestBucket {
        self.name_lookup().bucket(name)
    }

    pub fn proof(&self, name: impl AsRef<str>) -> ManifestProof {
        self.name_lookup().proof(name)
    }

    pub fn named_address(&self, name: impl AsRef<str>) -> ManifestAddress {
        self.name_lookup().named_address(name)
    }

    pub fn address_reservation(&self, name: impl AsRef<str>) -> ManifestAddressReservation {
        self.name_lookup().address_reservation(name)
    }

    /// Generates an unused bucket name with the given prefix.
    /// This should be used when you are programmatically generating buckets,
    /// and need to generate bucket names which do not clash.
    pub fn generate_bucket_name(&self, prefix: impl Into<String>) -> String {
        self.registrar.new_collision_free_bucket_name(prefix)
    }

    /// Generates an unused proof name with the given prefix.
    /// This should be used when you are programmatically generating proofs,
    /// and need to generate names which do not clash.
    pub fn generate_proof_name(&self, prefix: impl Into<String>) -> String {
        self.registrar.new_collision_free_proof_name(prefix)
    }

    /// Generates an unused address reservation name with the given prefix.
    /// This should be used when you are programmatically generating address reservations,
    /// and need to generate names which do not clash.
    pub fn generate_address_reservation_name(&self, prefix: impl Into<String>) -> String {
        self.registrar
            .new_collision_free_address_reservation_name(prefix)
    }

    /// Generates an unused address name with the given prefix.
    /// This should be used when you are programmatically generating named addresses,
    /// and need to generate names which do not clash.
    pub fn generate_address_name(&self, prefix: impl Into<String>) -> String {
        self.registrar
            .new_collision_free_address_reservation_name(prefix)
    }

    pub fn object_names(&self) -> KnownManifestObjectNames {
        self.registrar.object_names()
    }

    /// Example usage:
    /// ```
    /// # use radix_transactions::prelude::*;
    /// # use radix_engine_interface::prelude::*;
    /// # use radix_common::prelude::*;
    /// # let from_account_address = ComponentAddress::preallocated_account_from_public_key(
    /// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
    /// # );
    /// # let package_address = FAUCET_PACKAGE; // Just so it compiles
    ///
    /// let manifest = ManifestBuilder::new()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     // ...
    ///     .then(|mut builder| {
    ///         let code_blob_ref = builder.add_blob(vec![]);
    ///         builder
    ///             .call_function(
    ///                 package_address,
    ///                 "my_blueprint",
    ///                 "func_name",
    ///                 manifest_args!(code_blob_ref),
    ///             )
    ///     })
    ///     .build();
    /// ```
    pub fn add_blob(&mut self, blob_content: Vec<u8>) -> ManifestBlobRef {
        let hash = hash(&blob_content);
        self.manifest.add_blob(hash, blob_content);
        ManifestBlobRef(hash.0)
    }

    /// An internal method which is used by other methods - the callers are expected to handle
    /// registering buckets/proofs/etc and consuming them
    fn add_instruction(mut self, instruction: impl Into<M::Instruction>) -> Self {
        self.manifest.add_instruction(instruction.into());
        self
    }

    #[deprecated = "This should not be used apart from for test code purposefully constructing invalid manifests. Instead use the more-tailored instruction, or add_instruction_advanced."]
    pub fn add_raw_instruction_ignoring_all_side_effects(
        self,
        instruction: impl Into<M::Instruction>,
    ) -> Self {
        self.add_instruction(instruction)
    }

    /// Only for use in advanced use cases.
    /// Returns all the created symbols as part of the instruction.
    pub fn add_instruction_advanced(
        self,
        instruction: impl Into<M::Instruction>,
    ) -> (Self, NewSymbols) {
        self.add_instruction_advanced_internal(instruction.into())
    }

    /// Have an internal method to avoid monomorphization overhead
    fn add_instruction_advanced_internal(self, instruction: M::Instruction) -> (Self, NewSymbols) {
        let mut new_bucket = None;
        let mut new_proof = None;
        let mut new_address_reservation = None;
        let mut new_address_id = None;

        let registrar = &self.registrar;
        let lookup = self.name_lookup();

        match instruction.effect() {
            ManifestInstructionEffect::CreateBucket { .. } => {
                let bucket_name = registrar.new_collision_free_bucket_name("bucket");
                registrar.register_bucket(registrar.new_bucket(&bucket_name));
                new_bucket = Some(lookup.bucket(bucket_name));
            }
            ManifestInstructionEffect::CreateProof { .. }
            | ManifestInstructionEffect::CloneProof { .. } => {
                let proof_name = registrar.new_collision_free_proof_name("proof");
                registrar.register_proof(registrar.new_proof(&proof_name));
                new_proof = Some(lookup.proof(proof_name));
            }
            ManifestInstructionEffect::CreateAddressAndReservation { .. } => {
                let reservation_name =
                    registrar.new_collision_free_address_reservation_name("reservation");
                registrar.register_address_reservation(
                    registrar.new_address_reservation(&reservation_name),
                );

                let address_name = registrar.new_collision_free_address_name("address");
                registrar.register_named_address(registrar.new_named_address(&address_name));
                new_address_reservation = Some(lookup.address_reservation(reservation_name));
                new_address_id = Some(lookup.named_address_id(address_name));
            }
            ManifestInstructionEffect::ConsumeBucket { bucket, .. } => {
                registrar.consume_bucket(bucket);
            }
            ManifestInstructionEffect::ConsumeProof { proof, .. } => {
                registrar.consume_proof(proof);
            }
            ManifestInstructionEffect::DropManyProofs {
                drop_all_named_proofs,
                ..
            } => {
                if drop_all_named_proofs {
                    registrar.consume_all_proofs()
                }
            }
            // I've just noticed that this method doesn't consume things in the arguments of an invocation.
            // Ideally, much like the transaction validator, we should parse the included arguments
            // for things to consume and consume them. But this consumption is only used to catch errors
            // about re-use at manifest construction time.
            // And at present `add_instruction_advanced` is not actually used - so I'm not wasting time now
            // implementing this edge case.
            ManifestInstructionEffect::Invocation { .. } => {}
            ManifestInstructionEffect::ResourceAssertion { .. } => {}
        };

        (
            self.add_instruction(instruction),
            NewSymbols {
                new_bucket,
                new_proof,
                new_address_reservation,
                new_address_id,
            },
        )
    }

    /// Builds a transaction manifest.
    pub fn build(mut self) -> M {
        self.manifest.set_names(self.object_names().into());
        #[cfg(feature = "dump_manifest_to_file")]
        {
            let bytes = manifest_encode(&self.manifest).unwrap();
            let manifest_hash = hash(&bytes);
            let path = format!("manifest_{:?}.raw", manifest_hash);
            std::fs::write(&path, bytes).unwrap();
            println!("manifest dumped to file {}", &path);
        }
        self.manifest
    }
}

//===========================
// V1 Specific Methods
//===========================

impl<M: BuildableManifest> ManifestBuilder<M>
where
    M::Instruction: From<InstructionV1>,
{
    /// An internal method which is used by other methods - the callers are expected to handle
    /// registering buckets/proofs/etc and consuming them
    fn add_v1_instruction(self, instruction: impl Into<InstructionV1>) -> Self {
        self.add_instruction(instruction.into())
    }

    /// Takes resource from worktop.
    pub fn take_all_from_worktop(
        self,
        resource_address: impl ResolvableResourceAddress,
        new_bucket: impl NewManifestBucket,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_bucket.register(&self.registrar);
        self.add_v1_instruction(TakeAllFromWorktop { resource_address })
    }

    /// Takes resource from worktop, by amount.
    pub fn take_from_worktop(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
        new_bucket: impl NewManifestBucket,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        new_bucket.register(&self.registrar);
        self.add_v1_instruction(TakeFromWorktop {
            amount,
            resource_address,
        })
    }

    /// Takes resource from worktop, by non-fungible ids.
    pub fn take_non_fungibles_from_worktop(
        self,
        resource_address: impl ResolvableResourceAddress,
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
        new_bucket: impl NewManifestBucket,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_bucket.register(&self.registrar);
        self.add_v1_instruction(TakeNonFungiblesFromWorktop {
            ids: ids.into_iter().collect(),
            resource_address,
        })
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(self, bucket: impl ExistingManifestBucket) -> Self {
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_v1_instruction(ReturnToWorktop { bucket_id: bucket })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        self.add_v1_instruction(AssertWorktopContains {
            amount,
            resource_address,
        })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_any(
        self,
        resource_address: impl ResolvableResourceAddress,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        self.add_v1_instruction(AssertWorktopContainsAny { resource_address })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_non_fungibles(
        self,
        resource_address: impl ResolvableResourceAddress,
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        self.add_v1_instruction(AssertWorktopContainsNonFungibles {
            ids: ids.into_iter().collect(),
            resource_address,
        })
    }

    /// Pops the most recent proof from auth zone.
    pub fn pop_from_auth_zone(self, new_proof: impl NewManifestProof) -> Self {
        new_proof.register(&self.registrar);
        self.add_v1_instruction(PopFromAuthZone)
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(self, proof: impl ExistingManifestProof) -> Self {
        let proof = proof.mark_consumed(&self.registrar);
        self.add_v1_instruction(PushToAuthZone { proof_id: proof })
    }

    /// Creates proof from the auth zone by amount.
    pub fn create_proof_from_auth_zone_of_amount(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        new_proof.register(&self.registrar);
        self.add_v1_instruction(CreateProofFromAuthZoneOfAmount {
            amount,
            resource_address,
        })
    }

    /// Creates proof from the auth zone by non-fungible ids.
    pub fn create_proof_from_auth_zone_of_non_fungibles(
        self,
        resource_address: impl ResolvableResourceAddress,
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_v1_instruction(CreateProofFromAuthZoneOfNonFungibles {
            ids: ids.into_iter().collect(),
            resource_address,
        })
    }

    /// Creates proof from the auth zone
    pub fn create_proof_from_auth_zone_of_all(
        self,
        resource_address: impl ResolvableResourceAddress,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_v1_instruction(CreateProofFromAuthZoneOfAll { resource_address })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_amount(
        self,
        bucket: impl ExistingManifestBucket,
        amount: impl ResolvableDecimal,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        let amount = amount.resolve();
        new_proof.register(&self.registrar);
        self.add_v1_instruction(CreateProofFromBucketOfAmount {
            bucket_id: bucket,
            amount,
        })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_non_fungibles(
        self,
        bucket: impl ExistingManifestBucket,
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_v1_instruction(CreateProofFromBucketOfNonFungibles {
            bucket_id: bucket,
            ids: ids.into_iter().collect(),
        })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_all(
        self,
        bucket: impl ExistingManifestBucket,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_v1_instruction(CreateProofFromBucketOfAll { bucket_id: bucket })
    }

    /// Clones a proof.
    pub fn clone_proof(
        self,
        proof: impl ExistingManifestProof,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let proof = proof.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_v1_instruction(CloneProof { proof_id: proof })
    }

    pub fn allocate_global_address(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        new_address_reservation: impl NewManifestAddressReservation,
        new_address_name: impl Into<String>,
    ) -> Self {
        let package_address = package_address.resolve_static(&self.registrar);
        let blueprint_name = blueprint_name.into();
        let new_named_address = self.registrar.new_named_address(new_address_name);

        new_address_reservation.register(&self.registrar);
        self.registrar.register_named_address(new_named_address);
        self.add_v1_instruction(AllocateGlobalAddress {
            package_address,
            blueprint_name,
        })
    }

    /// Drops a proof.
    pub fn drop_proof(self, proof: impl ExistingManifestProof) -> Self {
        let proof = proof.mark_consumed(&self.registrar);
        self.add_v1_instruction(DropProof { proof_id: proof })
    }

    /// Drops all proofs.
    pub fn drop_all_proofs(self) -> Self {
        self.registrar.consume_all_proofs();
        self.add_v1_instruction(DropAllProofs)
    }

    /// Drops named proofs.
    pub fn drop_named_proofs(self) -> Self {
        self.registrar.consume_all_proofs();
        self.add_v1_instruction(DropNamedProofs)
    }

    /// Drops auth zone signature proofs.
    pub fn drop_auth_zone_signature_proofs(self) -> Self {
        self.add_v1_instruction(DropAuthZoneSignatureProofs)
    }

    /// Drops auth zone regular proofs.
    pub fn drop_auth_zone_regular_proofs(self) -> Self {
        self.add_v1_instruction(DropAuthZoneRegularProofs)
    }

    /// Drop auth zone proofs.
    pub fn drop_auth_zone_proofs(self) -> Self {
        self.add_v1_instruction(DropAuthZoneProofs)
    }

    /// Creates a fungible resource
    pub fn create_fungible_resource(
        self,
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        resource_roles: FungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Option<Decimal>,
    ) -> Self {
        let instruction = if let Some(initial_supply) = initial_supply {
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role,
                        divisibility,
                        track_total_supply,
                        metadata,
                        resource_roles,
                        initial_supply,
                        address_reservation: None,
                    }
                ),
            }
        } else {
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&FungibleResourceManagerCreateManifestInput {
                    owner_role,
                    divisibility,
                    track_total_supply,
                    metadata,
                    resource_roles,
                    address_reservation: None,
                }),
            }
        };
        self.add_v1_instruction(instruction)
    }

    /// Creates a new non-fungible resource
    pub fn create_non_fungible_resource<T, V>(
        self,
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Option<T>,
    ) -> Self
    where
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: ManifestEncode + NonFungibleData,
    {
        let instruction = if let Some(initial_supply) = initial_supply {
            let entries = initial_supply
                .into_iter()
                .map(|(id, e)| (id, (to_manifest_value_and_unwrap!(&e),)))
                .collect();

            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role,
                        id_type,
                        track_total_supply,
                        non_fungible_schema:
                            NonFungibleDataSchema::new_local_without_self_package_replacement::<V>(),
                        resource_roles,
                        metadata,
                        entries,
                        address_reservation: None,
                    }
                ),
            }
        } else {
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateManifestInput {
                        owner_role,
                        id_type,
                        track_total_supply,
                        non_fungible_schema:
                            NonFungibleDataSchema::new_local_without_self_package_replacement::<V>(),
                        resource_roles,
                        metadata,
                        address_reservation: None,
                    }
                ),
            }
        };

        self.add_v1_instruction(instruction)
    }

    pub fn create_ruid_non_fungible_resource<T, V>(
        self,
        owner_role: OwnerRole,
        track_total_supply: bool,
        metadata: ModuleConfig<MetadataInit>,
        resource_roles: NonFungibleResourceRoles,
        initial_supply: Option<T>,
    ) -> Self
    where
        T: IntoIterator<Item = V>,
        V: ManifestEncode + NonFungibleData,
    {
        let instruction = if let Some(initial_supply) = initial_supply {
            let entries = initial_supply
                .into_iter()
                .map(|e| (to_manifest_value_and_unwrap!(&e),))
                .collect();

            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                        owner_role,
                        track_total_supply,
                        non_fungible_schema:
                            NonFungibleDataSchema::new_local_without_self_package_replacement::<V>(),
                        resource_roles,
                        metadata,
                        entries,
                        address_reservation: None,
                    }
                ),
            }
        } else {
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                        owner_role,
                        track_total_supply,
                        non_fungible_schema:
                            NonFungibleDataSchema::new_local_without_self_package_replacement::<V>(),
                        resource_roles,
                        metadata,
                        entries: vec![],
                        address_reservation: None,
                    }
                ),
            }
        };

        self.add_v1_instruction(instruction)
    }

    pub fn update_non_fungible_data(
        self,
        resource_address: impl ResolvableResourceAddress,
        id: NonFungibleLocalId,
        field_name: impl Into<String>,
        data: impl ManifestEncode,
    ) -> Self {
        let address = resource_address.resolve(&self.registrar);
        let data = manifest_decode(&manifest_encode(&data).unwrap()).unwrap();
        self.call_method(
            address,
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
            NonFungibleResourceManagerUpdateDataManifestInput {
                id,
                field_name: field_name.into(),
                data,
            },
        )
    }

    pub fn create_identity_advanced(self, owner_role: OwnerRole) -> Self {
        self.add_v1_instruction(CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&IdentityCreateAdvancedInput { owner_role }),
        })
    }

    pub fn create_identity(self) -> Self {
        self.add_v1_instruction(CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&IdentityCreateInput {}),
        })
    }

    pub fn create_validator(
        self,
        key: Secp256k1PublicKey,
        fee_factor: impl ResolvableDecimal,
        xrd_payment: impl ExistingManifestBucket,
    ) -> Self {
        let fee_factor = fee_factor.resolve();
        let xrd_payment = xrd_payment.mark_consumed(&self.registrar);
        self.call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
            (key, fee_factor, xrd_payment),
        )
    }

    pub fn register_validator(self, validator_address: impl ResolvableComponentAddress) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.call_method(address, VALIDATOR_REGISTER_IDENT, ())
    }

    pub fn unregister_validator(self, validator_address: impl ResolvableComponentAddress) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.call_method(address, VALIDATOR_UNREGISTER_IDENT, ())
    }

    pub fn signal_protocol_update_readiness(
        self,
        validator_address: impl ResolvableComponentAddress,
        protocol_version_name: &str,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.call_method(
            address,
            VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS,
            (protocol_version_name.to_string(),),
        )
    }

    pub fn stake_validator_as_owner(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket: ManifestBucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_STAKE_AS_OWNER_IDENT, (bucket,))
    }

    pub fn stake_validator(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_STAKE_IDENT, (bucket,))
    }

    pub fn unstake_validator(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_UNSTAKE_IDENT, (bucket,))
    }

    pub fn claim_xrd(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_CLAIM_XRD_IDENT, (bucket,))
    }

    /// Calls a scrypto function where the arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// NOTE: If you need access to named buckets/proofs etc, use `then` or `call_function_with_name_lookup`
    /// instead.
    pub fn call_function(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        let package_address = package_address.resolve(&self.registrar);
        self.add_v1_instruction(CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args: arguments.resolve(),
        })
    }

    /// Calls a scrypto function where the arguments are a raw ManifestValue.
    /// The caller is required to ensure the ManifestValue is a Tuple.
    ///
    /// You should prefer `call_function` or `call_function_with_name_lookup` instead.
    pub fn call_function_raw(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments: ManifestValue,
    ) -> Self {
        let package_address = package_address.resolve(&self.registrar);
        self.add_v1_instruction(CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args: arguments,
        })
    }

    /// Calls a scrypto function where the arguments will be created using the given
    /// callback, which takes a `lookup` (allowing for resolving named buckets, proofs, etc)
    /// and returns resolvable arguments.
    ///
    /// You may prefer using `then` instead.
    ///
    /// The resolvable arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// Example:
    /// ```
    /// # use radix_transactions::prelude::*;
    /// # use radix_engine_interface::prelude::*;
    /// # use radix_common::prelude::*;
    /// # let package_address = RESOURCE_PACKAGE; // Just some address to get it to compile
    /// # let from_account_address = ComponentAddress::preallocated_account_from_public_key(
    /// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
    /// # );
    /// let manifest = ManifestBuilder::new()
    ///     .lock_fee_from_faucet()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     .take_from_worktop(XRD, dec!(1), "xrd_bucket")
    ///     .call_function_with_name_lookup(
    ///         package_address,
    ///         "SomeBlueprint",
    ///         "some_function",
    ///         |lookup| (
    ///             "argument1",
    ///             lookup.bucket("xrd_bucket"),
    ///             dec!("1.3")
    ///         ),
    ///     )
    ///     .build();
    ///
    /// // Alternative using `then`
    /// let manifest2 = ManifestBuilder::new()
    ///     .lock_fee_from_faucet()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     .take_from_worktop(XRD, dec!(1), "xrd_bucket")
    ///     .then(|builder| {
    ///         let lookup = builder.name_lookup();
    ///         builder.call_function(
    ///             package_address,
    ///             "SomeBlueprint",
    ///             "some_function",
    ///             ("argument1", lookup.bucket("xrd_bucket"), dec!("1.3")),
    ///         )
    ///     })
    ///     .build();
    /// ```
    pub fn call_function_with_name_lookup<T: ResolvableArguments>(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments_creator: impl FnOnce(&ManifestNameLookup) -> T,
    ) -> Self {
        let package_address = package_address.resolve(&self.registrar);
        let args = arguments_creator(&self.name_lookup()).resolve();

        self.add_v1_instruction(CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args,
        })
    }

    /// Calls a scrypto method where the arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// NOTE: If you need access to named buckets/proofs etc, use `call_method_with_name_lookup`
    /// instead.
    pub fn call_method(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        self.call_module_method(address, ModuleId::Main, method_name, arguments)
    }

    pub fn call_metadata_method(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        self.call_module_method(address, ModuleId::Metadata, method_name, arguments)
    }

    pub fn call_royalty_method(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        self.call_module_method(address, ModuleId::Royalty, method_name, arguments)
    }

    pub fn call_direct_access_method(
        self,
        address: InternalAddress,
        method_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        self.add_v1_instruction(CallDirectVaultMethod {
            address,
            method_name: method_name.into(),
            args: arguments.resolve(),
        })
    }

    pub fn set_owner_role(
        self,
        address: impl ResolvableGlobalAddress,
        rule: impl Into<AccessRule>,
    ) -> Self {
        self.call_module_method(
            address,
            ModuleId::RoleAssignment,
            ROLE_ASSIGNMENT_SET_OWNER_IDENT,
            RoleAssignmentSetOwnerInput { rule: rule.into() },
        )
    }

    pub fn lock_owner_role(self, address: impl ResolvableGlobalAddress) -> Self {
        self.call_module_method(
            address,
            ModuleId::RoleAssignment,
            ROLE_ASSIGNMENT_LOCK_OWNER_IDENT,
            RoleAssignmentLockOwnerInput {},
        )
    }

    pub fn set_main_role(
        self,
        address: impl ResolvableGlobalAddress,
        role_key: impl Into<RoleKey>,
        rule: impl Into<AccessRule>,
    ) -> Self {
        self.set_role(address, ModuleId::Main, role_key, rule)
    }

    pub fn set_role(
        self,
        address: impl ResolvableGlobalAddress,
        role_module: ModuleId,
        role_key: impl Into<RoleKey>,
        rule: impl Into<AccessRule>,
    ) -> Self {
        self.call_module_method(
            address,
            ModuleId::RoleAssignment,
            ROLE_ASSIGNMENT_SET_IDENT,
            RoleAssignmentSetInput {
                module: role_module,
                role_key: role_key.into(),
                rule: rule.into(),
            },
        )
    }

    pub fn get_role(
        self,
        address: impl ResolvableGlobalAddress,
        role_module: ModuleId,
        role_key: RoleKey,
    ) -> Self {
        self.call_module_method(
            address,
            ModuleId::RoleAssignment,
            ROLE_ASSIGNMENT_GET_IDENT,
            RoleAssignmentGetInput {
                module: role_module,
                role_key: role_key.into(),
            },
        )
    }

    pub fn call_role_assignment_method(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        self.call_module_method(address, ModuleId::RoleAssignment, method_name, arguments)
    }

    pub fn call_module_method(
        self,
        address: impl ResolvableGlobalAddress,
        module_id: ModuleId,
        method_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        match module_id {
            ModuleId::Main => self.add_v1_instruction(CallMethod {
                address,
                method_name: method_name.into(),
                args: arguments.resolve(),
            }),
            ModuleId::Metadata => self.add_v1_instruction(CallMetadataMethod {
                address,
                method_name: method_name.into(),
                args: arguments.resolve(),
            }),
            ModuleId::Royalty => self.add_v1_instruction(CallRoyaltyMethod {
                address,
                method_name: method_name.into(),
                args: arguments.resolve(),
            }),
            ModuleId::RoleAssignment => self.add_v1_instruction(CallRoleAssignmentMethod {
                address,
                method_name: method_name.into(),
                args: arguments.resolve(),
            }),
        }
    }

    /// Calls a scrypto method where the arguments are a raw ManifestValue.
    /// The caller is required to ensure the ManifestValue is a Tuple.
    ///
    /// You should prefer `call_function` or `call_function_with_name_lookup` instead.
    pub fn call_method_raw(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments: ManifestValue,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_v1_instruction(CallMethod {
            address,
            method_name: method_name.into(),
            args: arguments,
        })
    }

    /// Calls a scrypto method where the arguments will be created using the given
    /// callback, which takes a `lookup` (allowing for resolving named buckets, proofs, etc)
    /// and returns resolvable arguments.
    ///
    /// The resolvable arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// Example:
    /// ```
    /// # use radix_transactions::prelude::*;
    /// # use radix_engine_interface::prelude::*;
    /// # use radix_common::prelude::*;
    /// # let component_address = GENESIS_HELPER; // Just some address to get it to compile
    /// # let from_account_address = ComponentAddress::preallocated_account_from_public_key(
    /// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
    /// # );
    /// let manifest = ManifestBuilder::new()
    ///     .lock_fee_from_faucet()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     .take_from_worktop(XRD, dec!(1), "xrd_bucket")
    ///     .call_method_with_name_lookup(
    ///         component_address,
    ///         "some_method",
    ///         |lookup| (
    ///             "argument1",
    ///             lookup.bucket("xrd_bucket"),
    ///             dec!("1.3")
    ///         ),
    ///     )
    ///     .build();
    ///
    /// // Alternative using `then`
    /// let manifest2 = ManifestBuilder::new()
    ///     .lock_fee_from_faucet()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     .take_from_worktop(XRD, dec!(1), "xrd_bucket")
    ///     .then(|builder| {
    ///         let lookup = builder.name_lookup();
    ///         builder.call_method(
    ///             component_address,
    ///             "some_method",
    ///             ("argument1", lookup.bucket("xrd_bucket"), dec!("1.3")),
    ///         )
    ///     })
    ///     .build();
    /// ```
    pub fn call_method_with_name_lookup<T: ResolvableArguments>(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments_creator: impl FnOnce(&ManifestNameLookup) -> T,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        let args = arguments_creator(&self.name_lookup()).resolve();

        self.add_v1_instruction(CallMethod {
            address,
            method_name: method_name.into(),
            args,
        })
    }

    pub fn claim_package_royalties(self, package_address: impl ResolvablePackageAddress) -> Self {
        let address = package_address.resolve(&self.registrar);
        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackageClaimRoyaltiesInput {}),
        })
    }

    pub fn set_component_royalty(
        self,
        component_address: impl ResolvableComponentAddress,
        method: impl Into<String>,
        amount: RoyaltyAmount,
    ) -> Self {
        let address = component_address.resolve(&self.registrar);
        self.add_v1_instruction(CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentRoyaltySetInput {
                method: method.into(),
                amount,
            }),
        })
    }

    pub fn lock_component_royalty(
        self,
        component_address: impl ResolvableComponentAddress,
        method: impl Into<String>,
    ) -> Self {
        let address = component_address.resolve(&self.registrar);
        self.add_v1_instruction(CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentRoyaltyLockInput {
                method: method.into(),
            }),
        })
    }

    pub fn claim_component_royalties(
        self,
        component_address: impl ResolvableComponentAddress,
    ) -> Self {
        let address = component_address.resolve(&self.registrar);
        self.add_v1_instruction(CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentClaimRoyaltiesInput {}),
        })
    }

    pub fn set_metadata(
        self,
        address: impl ResolvableGlobalAddress,
        key: impl Into<String>,
        value: impl ToMetadataEntry,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        match value.to_metadata_entry() {
            Some(value) => self.add_v1_instruction(CallMetadataMethod {
                address: address.into(),
                method_name: METADATA_SET_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&MetadataSetInput {
                    key: key.into(),
                    value
                }),
            }),
            None => self.add_v1_instruction(CallMetadataMethod {
                address: address.into(),
                method_name: METADATA_REMOVE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&MetadataRemoveInput { key: key.into() }),
            }),
        }
    }

    pub fn lock_metadata(
        self,
        address: impl ResolvableGlobalAddress,
        key: impl Into<String>,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        let key = key.into();
        self.add_v1_instruction(CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_LOCK_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataLockInput { key }),
        })
    }

    pub fn freeze_metadata(
        self,
        address: impl ResolvableGlobalAddress,
        key: impl Into<String>,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_v1_instruction(CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_LOCK_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataLockInput { key: key.into() }),
        })
    }

    /// Publishes a package.
    pub fn publish_package_advanced(
        mut self,
        address_reservation: impl OptionalExistingManifestAddressReservation,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: impl Into<MetadataInit>,
        owner_role: OwnerRole,
    ) -> Self {
        let address_reservation = address_reservation.mark_consumed(&self.registrar);
        let code_blob_ref = self.add_blob(code);

        self.add_v1_instruction(CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                code: code_blob_ref,
                definition: definition,
                metadata: metadata.into(),
                package_address: address_reservation,
                owner_role,
            }),
        })
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package(mut self, code: Vec<u8>, definition: PackageDefinition) -> Self {
        let code_blob_ref = self.add_blob(code);

        self.add_v1_instruction(CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmManifestInput {
                code: code_blob_ref,
                definition,
                metadata: metadata_init!(),
            }),
        })
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package_with_owner(
        mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        owner_badge: NonFungibleGlobalId,
    ) -> Self {
        let code_blob_ref = self.add_blob(code);

        self.add_v1_instruction(CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                package_address: None,
                code: code_blob_ref,
                definition,
                metadata: metadata_init!(),
                owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
            }),
        })
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        self,
        metadata: ModuleConfig<MetadataInit>,
        owner_role: AccessRule,
    ) -> Self {
        self.create_fungible_resource(
            OwnerRole::Fixed(owner_role),
            true,
            18,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => OWNER;
                    minter_updater => OWNER;
                },
                burn_roles: burn_roles! {
                    burner => OWNER;
                    burner_updater => OWNER;
                },
                ..Default::default()
            },
            metadata,
            None,
        )
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: impl ResolvableDecimal,
    ) -> Self {
        let initial_supply = initial_supply.resolve();
        self.create_fungible_resource(
            owner_role,
            true,
            18,
            FungibleResourceRoles::default(),
            metadata,
            Some(initial_supply),
        )
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        self,
        metadata: ModuleConfig<MetadataInit>,
        owner_role: AccessRule,
    ) -> Self {
        self.create_fungible_resource(
            OwnerRole::Fixed(owner_role),
            false,
            0,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => OWNER;
                    minter_updater => OWNER;
                },
                burn_roles: burn_roles! {
                    burner => OWNER;
                    burner_updater => OWNER;
                },
                ..Default::default()
            },
            metadata,
            None,
        )
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: impl ResolvableDecimal,
    ) -> Self {
        let initial_supply = initial_supply.resolve();
        self.create_fungible_resource(
            owner_role,
            false,
            0,
            FungibleResourceRoles::default(),
            metadata,
            Some(initial_supply),
        )
    }

    pub fn burn_resource(self, bucket: impl ExistingManifestBucket) -> Self {
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_v1_instruction(BurnResource { bucket_id: bucket })
    }

    pub fn burn_from_worktop(
        self,
        amount: impl ResolvableDecimal,
        resource_address: impl ResolvableResourceAddress,
    ) -> Self {
        let amount = amount.resolve();
        let resource_address = resource_address.resolve(&self.registrar);

        let bucket = self.generate_bucket_name("to_burn");
        self.take_from_worktop(resource_address, amount, &bucket)
            .burn_resource(bucket)
    }

    pub fn burn_all_from_worktop(self, resource_address: impl ResolvableResourceAddress) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);

        let bucket = self.generate_bucket_name("to_burn");
        self.take_all_from_worktop(resource_address, &bucket)
            .burn_resource(bucket)
    }

    pub fn burn_non_fungible_from_worktop(
        self,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Self {
        let (resource_address, local_id) = non_fungible_global_id.into_parts();
        let bucket = self.generate_bucket_name("to_burn");

        self.take_non_fungibles_from_worktop(resource_address, [local_id], &bucket)
            .burn_resource(bucket)
    }

    pub fn mint_fungible(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = resource_address.resolve(&self.registrar);
        let amount = amount.resolve();
        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&FungibleResourceManagerMintInput { amount }),
        })
    }

    pub fn mint_non_fungible<T: IntoIterator<Item = (NonFungibleLocalId, V)>, V: ManifestEncode>(
        self,
        resource_address: impl ResolvableResourceAddress,
        entries: T,
    ) -> Self {
        let address = resource_address.resolve(&self.registrar);

        let entries = entries
            .into_iter()
            .map(|(id, e)| (id, (to_manifest_value_and_unwrap!(&e),)))
            .collect();

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerMintManifestInput {
                entries
            }),
        })
    }

    pub fn mint_ruid_non_fungible<T: IntoIterator<Item = V>, V: ManifestEncode>(
        self,
        resource_address: impl ResolvableResourceAddress,
        entries: T,
    ) -> Self {
        let address = resource_address.resolve(&self.registrar);

        let entries = entries
            .into_iter()
            .map(|e| (to_manifest_value_and_unwrap!(&e),))
            .collect();

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerMintRuidManifestInput {
                entries
            }),
        })
    }

    pub fn recall(self, vault_address: InternalAddress, amount: impl ResolvableDecimal) -> Self {
        let amount = amount.resolve();
        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_address,
            method_name: VAULT_RECALL_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultRecallInput { amount }),
        })
    }

    pub fn recall_non_fungibles(
        self,
        vault_address: InternalAddress,
        non_fungible_local_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let args = to_manifest_value_and_unwrap!(&NonFungibleVaultRecallNonFungiblesInput {
            non_fungible_local_ids: non_fungible_local_ids.into_iter().collect(),
        });

        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_address,
            method_name: NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
    }

    pub fn freeze_withdraw(self, vault_id: InternalAddress) -> Self {
        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::WITHDRAW,
            }),
        })
    }

    pub fn unfreeze_withdraw(self, vault_id: InternalAddress) -> Self {
        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::WITHDRAW,
            }),
        })
    }

    pub fn freeze_deposit(self, vault_id: InternalAddress) -> Self {
        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::DEPOSIT,
            }),
        })
    }

    pub fn unfreeze_deposit(self, vault_id: InternalAddress) -> Self {
        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::DEPOSIT,
            }),
        })
    }

    pub fn freeze_burn(self, vault_id: InternalAddress) -> Self {
        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::BURN,
            }),
        })
    }

    pub fn unfreeze_burn(self, vault_id: InternalAddress) -> Self {
        self.add_v1_instruction(CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::BURN,
            }),
        })
    }

    /// Creates an account.
    pub fn new_account_advanced(
        self,
        owner_role: OwnerRole,
        address_reservation: impl OptionalExistingManifestAddressReservation,
    ) -> Self {
        let address_reservation = address_reservation.mark_consumed(&self.registrar);

        self.add_v1_instruction(CallFunction {
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccountCreateAdvancedManifestInput {
                owner_role,
                address_reservation
            }),
        })
    }

    pub fn new_account(self) -> Self {
        self.add_v1_instruction(CallFunction {
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccountCreateInput {}),
        })
    }

    pub fn lock_fee_and_withdraw(
        self,
        account_address: impl ResolvableComponentAddress,
        amount_to_lock: impl ResolvableDecimal,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount_to_lock = amount_to_lock.resolve();
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeAndWithdrawInput {
            resource_address,
            amount,
            amount_to_lock,
        });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            args,
        })
    }

    pub fn lock_fee_and_withdraw_non_fungibles(
        self,
        account_address: impl ResolvableComponentAddress,
        amount_to_lock: impl ResolvableDecimal,
        resource_address: impl ResolvableResourceAddress,
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount_to_lock = amount_to_lock.resolve();
        let resource_address = resource_address.resolve_static(&self.registrar);

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeAndWithdrawNonFungiblesInput {
            amount_to_lock,
            resource_address,
            ids: ids.into_iter().collect(),
        });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
    }

    /// Locks a large fee from the faucet.
    pub fn lock_fee_from_faucet(self) -> Self {
        self.lock_standard_test_fee(FAUCET)
    }

    /// Locks a large fee from the XRD vault of an account.
    pub fn lock_standard_test_fee(self, account_address: impl ResolvableComponentAddress) -> Self {
        self.lock_fee(account_address, 5000)
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee(
        self,
        account_address: impl ResolvableComponentAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount = amount.resolve();

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeInput { amount });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            args,
        })
    }

    pub fn lock_contingent_fee(
        self,
        account_address: impl ResolvableComponentAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount = amount.resolve();
        let args = to_manifest_value_and_unwrap!(&AccountLockContingentFeeInput { amount });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            args,
        })
    }

    /// Locks a large fee from the faucet.
    pub fn get_free_xrd_from_faucet(self) -> Self {
        self.call_method(FAUCET, "free", ())
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        let args = to_manifest_value_and_unwrap!(&AccountWithdrawInput {
            resource_address,
            amount,
        });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_WITHDRAW_IDENT.to_string(),
            args,
        })
    }

    /// Withdraws a single non-fungible from an account.
    pub fn withdraw_non_fungible_from_account(
        self,
        account_address: impl ResolvableComponentAddress,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Self {
        let (resource_address, local_id) = non_fungible_global_id.into_parts();
        self.withdraw_non_fungibles_from_account(account_address, resource_address, [local_id])
    }

    /// Withdraws non-fungibles from an account.
    pub fn withdraw_non_fungibles_from_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);

        let args = to_manifest_value_and_unwrap!(&AccountWithdrawNonFungiblesInput {
            ids: ids.into_iter().collect(),
            resource_address,
        });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
    }

    /// Withdraws resource from an account.
    pub fn burn_in_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        let args = to_manifest_value_and_unwrap!(&AccountBurnInput {
            resource_address,
            amount
        });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_BURN_IDENT.to_string(),
            args,
        })
    }

    /// Burns a single non-fungible from an account.
    pub fn burn_non_fungible_in_account(
        self,
        account_address: impl ResolvableComponentAddress,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Self {
        let (resource_address, local_id) = non_fungible_global_id.into_parts();
        self.burn_non_fungibles_in_account(account_address, resource_address, [local_id])
    }

    /// Burns non-fungibles from an account.
    pub fn burn_non_fungibles_in_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        local_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let account_address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);

        self.call_method(
            account_address,
            ACCOUNT_BURN_NON_FUNGIBLES_IDENT,
            AccountBurnNonFungiblesInput {
                resource_address,
                ids: local_ids.into_iter().collect(),
            },
        )
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_amount(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        let args = to_manifest_value_and_unwrap!(&AccountCreateProofOfAmountInput {
            resource_address,
            amount,
        });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            args,
        })
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_non_fungible(
        self,
        account_address: impl ResolvableComponentAddress,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Self {
        let (resource_address, local_id) = non_fungible_global_id.into_parts();
        self.create_proof_from_account_of_non_fungibles(
            account_address,
            resource_address,
            [local_id],
        )
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_non_fungibles(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        local_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);

        let args = to_manifest_value_and_unwrap!(&AccountCreateProofOfNonFungiblesInput {
            resource_address,
            ids: local_ids.into_iter().collect(),
        });

        self.add_v1_instruction(CallMethod {
            address: address.into(),
            method_name: ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
    }

    pub fn deposit(
        self,
        account_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        let bucket = bucket.mark_consumed(&self.registrar);

        self.call_method(address, ACCOUNT_DEPOSIT_IDENT, manifest_args!(bucket))
    }

    /// Note - the batch should either be:
    /// * `ManifestExpression::EntireWorktop`,
    /// * An array, vec, or btreeset of bucket names or ManifestBuckets, eg `["my_bucket_1", "my_bucket_2"]`
    /// * An empty, explicitly typed array of strings, eg `Vec::<String>::new()`
    pub fn deposit_batch(
        self,
        account_address: impl ResolvableComponentAddress,
        batch: impl ResolvableBucketBatch,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let batch = batch.consume_and_resolve(&self.registrar);

        self.call_method(address, ACCOUNT_DEPOSIT_BATCH_IDENT, manifest_args!(batch))
    }

    pub fn deposit_entire_worktop(self, account_address: impl ResolvableComponentAddress) -> Self {
        self.deposit_batch(account_address, ManifestExpression::EntireWorktop)
    }

    pub fn try_deposit_or_abort(
        self,
        account_address: impl ResolvableComponentAddress,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        let bucket = bucket.mark_consumed(&self.registrar);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
            manifest_args!(bucket, authorized_depositor_badge),
        )
    }

    /// Note - the batch should either be:
    /// * `ManifestExpression::EntireWorktop`,
    /// * An array, vec, or btreeset of bucket names or ManifestBuckets, eg `["my_bucket_1", "my_bucket_2"]`
    /// * An empty, explicitly typed array of strings, eg `Vec::<String>::new()`
    pub fn try_deposit_batch_or_abort(
        self,
        account_address: impl ResolvableComponentAddress,
        batch: impl ResolvableBucketBatch,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let batch = batch.consume_and_resolve(&self.registrar);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(batch, authorized_depositor_badge),
        )
    }

    pub fn try_deposit_entire_worktop_or_abort(
        self,
        account_address: impl ResolvableComponentAddress,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
    ) -> Self {
        self.try_deposit_batch_or_abort(
            account_address,
            ManifestExpression::EntireWorktop,
            authorized_depositor_badge,
        )
    }

    pub fn try_deposit_or_refund(
        self,
        account_address: impl ResolvableComponentAddress,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        let bucket = bucket.mark_consumed(&self.registrar);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
            manifest_args!(bucket, authorized_depositor_badge),
        )
    }

    /// Note - the batch should either be:
    /// * `ManifestExpression::EntireWorktop`,
    /// * An array, vec, or btreeset of bucket names or ManifestBuckets, eg `["my_bucket_1", "my_bucket_2"]`
    /// * An empty, explicitly typed array of strings, eg `Vec::<String>::new()`
    pub fn try_deposit_batch_or_refund(
        self,
        account_address: impl ResolvableComponentAddress,
        batch: impl ResolvableBucketBatch,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let batch = batch.consume_and_resolve(&self.registrar);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
            manifest_args!(batch, authorized_depositor_badge),
        )
    }

    pub fn try_deposit_entire_worktop_or_refund(
        self,
        account_address: impl ResolvableComponentAddress,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
    ) -> Self {
        self.try_deposit_batch_or_refund(
            account_address,
            ManifestExpression::EntireWorktop,
            authorized_depositor_badge,
        )
    }

    pub fn create_access_controller(
        self,
        controlled_asset: impl ExistingManifestBucket,
        primary_role: AccessRule,
        recovery_role: AccessRule,
        confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> Self {
        let controlled_asset = controlled_asset.mark_consumed(&self.registrar);
        self.call_function(
            ACCESS_CONTROLLER_PACKAGE,
            ACCESS_CONTROLLER_BLUEPRINT,
            ACCESS_CONTROLLER_CREATE_IDENT,
            (
                controlled_asset,
                RuleSet {
                    primary_role,
                    recovery_role,
                    confirmation_role,
                },
                timed_recovery_delay_in_minutes,
                Option::<()>::None,
            ),
        )
    }
}

//===========================
// V2 Specific Methods
//===========================

impl<M: BuildableManifest> ManifestBuilder<M>
where
    M::Instruction: From<InstructionV2>,
{
    /// An internal method which is used by other methods - the callers are expected to handle
    /// registering buckets/proofs/etc and consuming them
    fn add_v2_instruction(self, instruction: impl Into<InstructionV2>) -> Self {
        self.add_instruction(instruction.into())
    }

    pub fn yield_to_parent(self, arguments: impl ResolvableArguments) -> Self {
        self.add_v2_instruction(YieldToParent {
            args: arguments.resolve(),
        })
    }

    // TODO: Replace ManifestIntent with impl ExistingManifestIntent, and have some way to register a child.
    pub fn yield_to_child(
        self,
        child_manifest_intent: ManifestIntent,
        arguments: impl ResolvableArguments,
    ) -> Self {
        self.add_v2_instruction(YieldToChild {
            child_index: child_manifest_intent,
            args: arguments.resolve(),
        })
    }

    pub fn verify_parent(self, access_rule: impl ResolvableArguments) -> Self {
        self.add_v2_instruction(VerifyParent {
            access_rule: access_rule.resolve(),
        })
    }
}

impl Default for ManifestBuilder<TransactionManifestV1> {
    fn default() -> Self {
        ManifestBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_builder_and_bucket_and_proof() -> (ManifestBuilder, ManifestBucket, ManifestProof) {
        let builder = ManifestBuilder::new()
            .take_from_worktop(XRD, dec!(100), "bucket")
            .create_proof_from_bucket_of_amount("bucket", dec!(5), "proof");
        let lookup = builder.name_lookup();
        let proof_id = lookup.proof("proof");
        let bucket_id = lookup.bucket("bucket");
        (builder, bucket_id, proof_id)
    }

    #[test]
    fn test_manifest_builder_add_instruction_advanced_proof() {
        let (builder, _, proof_id) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(CloneProof { proof_id });

        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(PopFromAuthZone);

        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(CreateProofFromAuthZoneOfAmount {
            resource_address: XRD,
            amount: dec!(1),
        });

        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(CreateProofFromAuthZoneOfNonFungibles {
            resource_address: XRD,
            ids: vec![],
        });

        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(CreateProofFromAuthZoneOfAll {
            resource_address: XRD,
        });

        let (builder, bucket_id, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(CreateProofFromBucketOfAmount {
            bucket_id,
            amount: dec!(1),
        });

        let (builder, bucket_id, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(CreateProofFromBucketOfNonFungibles {
            bucket_id,
            ids: vec![],
        });

        let (builder, bucket_id, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(CreateProofFromBucketOfAll { bucket_id });
    }

    #[test]
    fn test_manifest_builder_add_instruction_advanced_worktop() {
        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(TakeFromWorktop {
            resource_address: XRD,
            amount: dec!(1),
        });

        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(TakeAllFromWorktop {
            resource_address: XRD,
        });

        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(TakeNonFungiblesFromWorktop {
            resource_address: XRD,
            ids: vec![],
        });
    }

    #[test]
    fn test_manifest_builder_add_instruction_advanced_global_address() {
        let (builder, _, _) = get_builder_and_bucket_and_proof();
        builder.add_instruction_advanced(AllocateGlobalAddress {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
        });
    }

    #[test]
    fn test_manifest_builder_complex_deposit_batch_build_process_works() {
        let account = GENESIS_HELPER; // Not actually an account, but not relevant for this test
        ManifestBuilder::new()
            .get_free_xrd_from_faucet()
            .take_from_worktop(XRD, dec!(1000), "bucket_1")
            .try_deposit_entire_worktop_or_abort(account, None)
            .try_deposit_batch_or_abort(account, ["bucket_1"], None)
            .build();
    }
}
