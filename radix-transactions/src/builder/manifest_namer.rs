use crate::internal_prelude::*;

/// This is used by a user to lookup buckets/proofs/reservations/addresses
/// for working with a manifest builder.
pub struct ManifestNameLookup {
    core: Rc<RefCell<ManifestNamerCore>>,
}

/// This is used by a manifest builder.
///
/// It shares a core with a ManifestNameLookup.
///
/// It offers more options than a ManifestNamer, to allow for the manifest instructions
/// to control the association of names (eg `NewManifestBucket`) with the corresponding ids
/// (eg `ManifestBucket`).
pub struct ManifestNameRegistrar {
    core: Rc<RefCell<ManifestNamerCore>>,
}

/// The ManifestNamerId is a mechanism to try to avoid people accidentally mismatching namers and builders
/// Such a mismatch would create unexpected behaviour
#[derive(PartialEq, Eq, Clone, Copy, Default)]
struct ManifestNamerId(u64);

static GLOBAL_INCREMENTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl ManifestNamerId {
    pub fn new_unique() -> Self {
        Self(GLOBAL_INCREMENTER.fetch_add(1, core::sync::atomic::Ordering::Acquire))
    }
}

/// This exposes a shared stateful core between a ManifestNamer and its corresponding Registrar.
/// This core is wrapped in a Rc<RefCell> for sharing between the two.
#[derive(Default)]
struct ManifestNamerCore {
    namer_id: ManifestNamerId,
    id_allocator: ManifestIdAllocator,
    named_buckets: IndexMap<String, ManifestObjectState<ManifestBucket>>,
    named_proofs: IndexMap<String, ManifestObjectState<ManifestProof>>,
    named_addresses: NonIterMap<String, ManifestObjectState<ManifestNamedAddress>>,
    named_address_reservations: NonIterMap<String, ManifestObjectState<ManifestAddressReservation>>,
    named_intents: NonIterMap<String, ManifestObjectState<ManifestNamedIntent>>,
    object_names: KnownManifestObjectNames,
}

impl ManifestNamerCore {
    pub fn new_named_bucket(&mut self, name: impl Into<String>) -> NamedManifestBucket {
        let name = name.into();
        let old_entry = self
            .named_buckets
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new bucket with the same name \"{name}\" multiple times");
        }
        NamedManifestBucket {
            namer_id: self.namer_id,
            name,
        }
    }

    pub fn new_collision_free_bucket_name(&mut self, prefix: &str) -> String {
        for name_counter in 1..u32::MAX {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            if !self.named_buckets.contains_key(&name) {
                return name;
            }
        }
        panic!("Did not resolve a name");
    }

    pub fn resolve_named_bucket(&self, name: impl AsRef<str>) -> ManifestBucket {
        match self.named_buckets.get(name.as_ref()) {
            Some(ManifestObjectState::Present(bucket)) => bucket.clone(),
            Some(ManifestObjectState::Consumed) => panic!("Bucket with name \"{}\" has already been consumed", name.as_ref()),
            _ => panic!("You cannot use a bucket with name \"{}\" before it has been created with a relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    /// This is intended for registering a bucket name to an allocated identifier, as part of processing a manifest
    /// instruction which creates a bucket.
    pub fn register_bucket(&mut self, new: NamedManifestBucket) {
        if self.namer_id != new.namer_id {
            panic!("NewManifestBucket cannot be registered against a different ManifestNamer")
        }
        let new_bucket = self.id_allocator.new_bucket_id();
        match self.named_buckets.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_bucket);
                self
                .object_names.bucket_names.insert(new_bucket, new.name);
            },
            Some(_) => unreachable!("NewManifestBucket was somehow registered twice"),
            None => unreachable!("NewManifestBucket was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    pub fn consume_bucket(&mut self, consumed: ManifestBucket) {
        let name = self
            .object_names
            .bucket_names
            .get(&consumed)
            .expect("Consumed bucket was not recognised")
            .to_string();
        let entry = self
            .named_buckets
            .get_mut(&name)
            .expect("Inverse index somehow became inconsistent");
        *entry = ManifestObjectState::Consumed;
    }

    pub fn consume_all_buckets(&mut self) {
        for (_, state) in self.named_buckets.iter_mut() {
            if let ManifestObjectState::Present(_) = state {
                *state = ManifestObjectState::Consumed;
            }
        }
    }

    pub fn assert_bucket_exists(&self, bucket: ManifestBucket) {
        self.object_names
            .bucket_names
            .get(&bucket)
            .expect("Bucket was not recognised - perhaps you're using a bucket not sourced from this builder?");
    }

    pub fn new_named_proof(&mut self, name: impl Into<String>) -> NamedManifestProof {
        let name = name.into();
        let old_entry = self
            .named_proofs
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new proof with the same name \"{name}\" multiple times");
        }
        NamedManifestProof {
            namer_id: self.namer_id,
            name,
        }
    }

    pub fn new_collision_free_proof_name(&mut self, prefix: &str) -> String {
        for name_counter in 1..u32::MAX {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            if !self.named_proofs.contains_key(&name) {
                return name;
            }
        }
        panic!("Did not resolve a name");
    }

    pub fn resolve_named_proof(&self, name: impl AsRef<str>) -> ManifestProof {
        match self.named_proofs.get(name.as_ref()) {
            Some(ManifestObjectState::Present(proof)) => proof.clone(),
            Some(ManifestObjectState::Consumed) => panic!("Proof with name \"{}\" has already been consumed", name.as_ref()),
            _ => panic!("You cannot use a proof with name \"{}\" before it has been created with a relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    /// This is intended for registering a proof name to an allocated identifier, as part of processing a manifest
    /// instruction which creates a proof.
    pub fn register_proof(&mut self, new: NamedManifestProof) {
        if self.namer_id != new.namer_id {
            panic!("NewManifestProof cannot be registered against a different ManifestNamer")
        }
        let new_proof = self.id_allocator.new_proof_id();
        match self.named_proofs.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_proof);
                self
                .object_names.proof_names.insert(new_proof, new.name);
            },
            Some(_) => unreachable!("NewManifestProof was somehow registered twice"),
            None => unreachable!("NewManifestProof was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    pub fn consume_proof(&mut self, consumed: ManifestProof) {
        let name = self
            .object_names
            .proof_names
            .get(&consumed)
            .expect("Consumed proof was not recognised")
            .to_string();
        let entry = self
            .named_proofs
            .get_mut(&name)
            .expect("Inverse index somehow became inconsistent");
        *entry = ManifestObjectState::Consumed;
    }

    pub fn consume_all_proofs(&mut self) {
        for (_, state) in self.named_proofs.iter_mut() {
            if let ManifestObjectState::Present(_) = state {
                *state = ManifestObjectState::Consumed;
            }
        }
    }

    pub fn assert_proof_exists(&self, proof: ManifestProof) {
        self.object_names
            .proof_names
            .get(&proof)
            .expect("Proof was not recognised - perhaps you're using a proof not sourced from this builder?");
    }

    pub fn new_named_address_reservation(
        &mut self,
        name: impl Into<String>,
    ) -> NamedManifestAddressReservation {
        let name = name.into();
        let old_entry = self
            .named_address_reservations
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new address reservation with the same name \"{name}\" multiple times");
        }
        NamedManifestAddressReservation {
            namer_id: self.namer_id,
            name,
        }
    }

    pub fn new_collision_free_address_reservation_name(&mut self, prefix: &str) -> String {
        for name_counter in 1..u32::MAX {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            if !self.named_address_reservations.contains_key(&name) {
                return name;
            }
        }
        panic!("Did not resolve a name");
    }

    pub fn resolve_named_address_reservation(
        &self,
        name: impl AsRef<str>,
    ) -> ManifestAddressReservation {
        match self.named_address_reservations.get(name.as_ref()) {
            Some(ManifestObjectState::Present(address_reservation)) => address_reservation.clone(),
            Some(ManifestObjectState::Consumed) => panic!("Address reservation with name \"{}\" has already been consumed", name.as_ref()),
            _ => panic!("You cannot use an address reservation with name \"{}\" before it has been created with a relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    /// This is intended for registering an address reservation to an allocated identifier, as part of processing a manifest
    /// instruction which creates an address reservation.
    pub fn register_address_reservation(&mut self, new: NamedManifestAddressReservation) {
        if self.namer_id != new.namer_id {
            panic!("NewManifestAddressReservation cannot be registered against a different ManifestNamer")
        }
        let new_address_reservation = self.id_allocator.new_address_reservation_id();
        match self.named_address_reservations.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_address_reservation);
                self
                .object_names.address_reservation_names.insert(new_address_reservation, new.name);
            },
            Some(_) => unreachable!("NewManifestAddressReservation was somehow registered twice"),
            None => unreachable!("NewManifestAddressReservation was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    pub fn consume_address_reservation(&mut self, consumed: ManifestAddressReservation) {
        let name = self
            .object_names
            .address_reservation_names
            .get(&consumed)
            .expect("Consumed address reservation was not recognised")
            .to_string();
        let entry = self
            .named_address_reservations
            .get_mut(&name)
            .expect("Inverse index somehow became inconsistent");
        *entry = ManifestObjectState::Consumed;
    }

    pub fn assert_address_reservation_exists(
        &self,
        address_reservation: ManifestAddressReservation,
    ) {
        self.object_names
            .address_reservation_names
            .get(&address_reservation)
            .expect("Address reservation was not recognised - perhaps you're using an address reservation not sourced from this builder?");
    }

    pub fn new_named_address(&mut self, name: impl Into<String>) -> NamedManifestAddress {
        let name = name.into();
        let old_entry = self
            .named_addresses
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new named address with the same name \"{name}\" multiple times");
        }
        NamedManifestAddress {
            namer_id: self.namer_id,
            name,
        }
    }

    pub fn new_collision_free_address_name(&mut self, prefix: &str) -> String {
        for name_counter in 1..u32::MAX {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            if !self.named_addresses.contains_key(&name) {
                return name;
            }
        }
        panic!("Did not resolve a name");
    }

    pub fn resolve_named_address(&self, name: impl AsRef<str>) -> ManifestNamedAddress {
        match self.named_addresses.get(name.as_ref()) {
            Some(ManifestObjectState::Present(address)) => address.clone(),
            Some(ManifestObjectState::Consumed) => unreachable!("Address not consumable"),
            _ => panic!("You cannot use a named address with name \"{}\" before it has been created with a relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    /// This is intended for registering an address reservation to an allocated identifier, as part of processing a manifest
    /// instruction which creates a named address.
    pub fn register_named_address(&mut self, new: NamedManifestAddress) {
        if self.namer_id != new.namer_id {
            panic!("NewManifestNamedAddress cannot be registered against a different ManifestNamer")
        }
        let address_id = self.id_allocator.new_address_id();
        match self.named_addresses.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(address_id);
                self
                .object_names.address_names.insert(address_id, new.name);
            },
            Some(_) => unreachable!("NewManifestNamedAddress was somehow registered twice"),
            None => unreachable!("NewManifestNamedAddress was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    pub fn assert_named_address_exists(&self, named_address: ManifestNamedAddress) {
        self.object_names
            .address_names
            .get(&named_address)
            .expect("Address was not recognised - perhaps you're using a named address not sourced from this builder?");
    }

    // Intent
    pub fn new_intent(&mut self, name: impl Into<String>) -> NamedManifestIntent {
        let name = name.into();
        let old_entry = self
            .named_intents
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!(
                "You cannot create a new named intent with the same name \"{name}\" multiple times"
            );
        }
        NamedManifestIntent {
            namer_id: self.namer_id,
            name,
        }
    }

    pub fn new_collision_free_intent_name(&mut self, prefix: &str) -> String {
        for name_counter in 1..u32::MAX {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            if !self.named_intents.contains_key(&name) {
                return name;
            }
        }
        panic!("Did not resolve a name");
    }

    pub fn resolve_intent(&self, name: impl AsRef<str>) -> ManifestNamedIntent {
        match self.named_intents.get(name.as_ref()) {
            Some(ManifestObjectState::Present(id)) => *id,
            Some(ManifestObjectState::Consumed) => unreachable!("Intent binding has already been consumed"),
            _ => panic!("\nAn intent with name \"{}\" has been referenced before it has been registered. To register:\n * If using a transaction builder, first use `add_signed_child(\"{}\", signed_partial_transaction)` to register the child, and then use `manifest_builder(|builder| builder...)` to build the manifest, which automatically adds the `use_child` lines at the start of the manifest.\n * If using the manifest builder by itself, use `manifest_builder.use_child(\"{}\", root_subintent_hash)` at the start to register the child subintent.\n", name.as_ref(), name.as_ref(), name.as_ref()),
        }
    }

    /// This is intended for registering an address reservation to an allocated identifier, as part of processing a manifest
    /// instruction which creates a named address.
    pub fn register_intent(&mut self, new: NamedManifestIntent) {
        if self.namer_id != new.namer_id {
            panic!("NamedManifestIntent cannot be registered against a different ManifestNamer")
        }
        let id = self.id_allocator.new_named_intent_id();
        match self.named_intents.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(id);
                self
                .object_names.intent_names.insert(id, new.name);
            },
            Some(_) => unreachable!("NamedManifestIntent was somehow registered twice"),
            None => unreachable!("NamedManifestIntent was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    pub fn assert_intent_exists(&self, intent: impl Into<ManifestNamedIntent>) {
        self.object_names
            .intent_names
            .get(&intent.into())
            .expect("Named intent was not recognised - perhaps you're using a named intent not sourced from this builder?");
    }
}

impl ManifestNameLookup {
    pub fn bucket(&self, name: impl AsRef<str>) -> ManifestBucket {
        self.core.borrow().resolve_named_bucket(name)
    }

    pub fn proof(&self, name: impl AsRef<str>) -> ManifestProof {
        self.core.borrow().resolve_named_proof(name)
    }

    pub fn address_reservation(&self, name: impl AsRef<str>) -> ManifestAddressReservation {
        self.core.borrow().resolve_named_address_reservation(name)
    }

    pub fn named_address(&self, name: impl AsRef<str>) -> ManifestNamedAddress {
        self.core.borrow().resolve_named_address(name)
    }

    pub fn intent(&self, name: impl AsRef<str>) -> ManifestNamedIntent {
        self.core.borrow().resolve_intent(name)
    }
}

impl ManifestNameRegistrar {
    pub fn new() -> Self {
        Self {
            core: Rc::new(RefCell::new(ManifestNamerCore {
                namer_id: ManifestNamerId::new_unique(),
                ..Default::default()
            })),
        }
    }

    pub fn name_lookup(&self) -> ManifestNameLookup {
        ManifestNameLookup {
            core: self.core.clone(),
        }
    }

    /// This just registers a string name.
    /// It's not yet bound to anything until `register_bucket` is called.
    pub fn new_bucket(&self, name: impl Into<String>) -> NamedManifestBucket {
        self.core.borrow_mut().new_named_bucket(name)
    }

    pub fn new_collision_free_bucket_name(&self, prefix: impl Into<String>) -> String {
        self.core
            .borrow_mut()
            .new_collision_free_bucket_name(&prefix.into())
    }

    /// This is intended for registering a bucket name to an allocated identifier, as part of processing a manifest
    /// instruction which creates a bucket.
    pub fn register_bucket(&self, new: NamedManifestBucket) {
        self.core.borrow_mut().register_bucket(new);
    }

    pub fn consume_bucket(&self, consumed: ManifestBucket) {
        self.core.borrow_mut().consume_bucket(consumed);
    }

    pub fn consume_all_buckets(&self) {
        self.core.borrow_mut().consume_all_buckets();
    }

    /// This just registers a string name.
    /// It's not yet bound to anything until `register_proof` is called.
    pub fn new_proof(&self, name: impl Into<String>) -> NamedManifestProof {
        self.core.borrow_mut().new_named_proof(name)
    }

    pub fn new_collision_free_proof_name(&self, prefix: impl Into<String>) -> String {
        self.core
            .borrow_mut()
            .new_collision_free_proof_name(&prefix.into())
    }

    /// This is intended for registering a proof name to an allocated identifier, as part of processing a manifest
    /// instruction which creates a proof.
    pub fn register_proof(&self, new: NamedManifestProof) {
        self.core.borrow_mut().register_proof(new)
    }

    pub fn consume_proof(&self, consumed: ManifestProof) {
        self.core.borrow_mut().consume_proof(consumed)
    }

    pub fn consume_all_proofs(&self) {
        self.core.borrow_mut().consume_all_proofs()
    }

    /// This just registers a string name.
    /// It's not yet bound to anything until `register_address_reservation` is called.
    pub fn new_address_reservation(
        &self,
        name: impl Into<String>,
    ) -> NamedManifestAddressReservation {
        self.core.borrow_mut().new_named_address_reservation(name)
    }

    pub fn new_collision_free_address_reservation_name(&self, prefix: impl Into<String>) -> String {
        self.core
            .borrow_mut()
            .new_collision_free_address_reservation_name(&prefix.into())
    }

    /// This is intended for registering an address reservation to an allocated identifier, as part of processing a manifest
    /// instruction which creates an address reservation.
    pub fn register_address_reservation(&self, new: NamedManifestAddressReservation) {
        self.core.borrow_mut().register_address_reservation(new);
    }

    pub fn consume_address_reservation(&self, consumed: ManifestAddressReservation) {
        self.core.borrow_mut().consume_address_reservation(consumed);
    }

    /// This just registers a string name.
    /// It's not yet bound to anything until `register_named_address` is called.
    pub fn new_named_address(&self, name: impl Into<String>) -> NamedManifestAddress {
        self.core.borrow_mut().new_named_address(name)
    }

    pub fn new_collision_free_address_name(&self, prefix: impl Into<String>) -> String {
        self.core
            .borrow_mut()
            .new_collision_free_address_name(&prefix.into())
    }

    /// This is intended for registering an address reservation to an allocated identifier, as part of processing a manifest
    /// instruction which creates a named address.
    pub fn register_named_address(&self, new: NamedManifestAddress) {
        self.core.borrow_mut().register_named_address(new)
    }

    /// This just registers a string name.
    /// It's not yet bound to anything until `register_intent` is called.
    pub fn new_intent(&self, name: impl Into<String>) -> NamedManifestIntent {
        self.core.borrow_mut().new_intent(name)
    }

    pub fn new_collision_free_intent_name(&self, prefix: impl Into<String>) -> String {
        self.core
            .borrow_mut()
            .new_collision_free_intent_name(&prefix.into())
    }

    /// This is intended for registering an intent to an allocated identifier, as part of processing a manifest
    /// instruction which creates a named intent.
    pub fn register_intent(&self, new: NamedManifestIntent) {
        self.core.borrow_mut().register_intent(new)
    }

    pub fn check_intent_exists(&self, intent: impl Into<ManifestNamedIntent>) {
        self.core.borrow().assert_intent_exists(intent)
    }

    pub fn object_names(&self) -> KnownManifestObjectNames {
        self.core.borrow().object_names.clone()
    }
}

pub enum ManifestObjectState<T> {
    Unregistered,
    Present(T),
    Consumed,
}

//=====================
// BUCKET
//=====================

impl LabelResolver<ManifestBucket> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> ManifestBucket {
        self.name_lookup().bucket(name)
    }
}

/// Represents a new [`ManifestBucket`] which needs registering.
#[must_use]
pub struct NamedManifestBucket {
    namer_id: ManifestNamerId,
    name: String,
}

labelled_resolvable_with_identity_impl!(NamedManifestBucket, resolver_output: Self);

impl LabelResolver<NamedManifestBucket> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> NamedManifestBucket {
        self.new_bucket(name)
    }
}

/// Binds a name for a new [`ManifestBucket`].
///
/// Accepts a string representing the name to use for the bucket,
/// or a newly created bucket from a [`ManifestNameRegistrar`].
pub trait NewManifestBucket {
    fn register(self, registrar: &ManifestNameRegistrar);
}

impl<T: LabelledResolve<NamedManifestBucket>> NewManifestBucket for T {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_bucket(self.labelled_resolve(registrar));
    }
}

/// An existing bucket. Its handle will NOT be consumed by this instruction.
///
/// Accepts a string referencing the name of an existing created bucket,
/// or an existing bucket from a [`ManifestNameLookup`].
pub trait ReferencedManifestBucket {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestBucket;
}

impl<T: LabelledResolve<ManifestBucket>> ReferencedManifestBucket for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestBucket {
        let bucket = self.labelled_resolve(registrar);
        registrar.core.borrow().assert_bucket_exists(bucket);
        bucket
    }
}

/// A bucket which whose name/handle will be marked as used by this action.
///
/// This doesn't necessarily mean that the underlying bucket will be dropped,
/// but if its name/handle in the manifest can no longer be used.
///
/// Accepts a string referencing the name of an existing created bucket,
/// or an existing bucket from a [`ManifestNameLookup`].
pub trait ConsumedManifestBucket {
    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestBucket;
}

impl<T: LabelledResolve<ManifestBucket>> ConsumedManifestBucket for T {
    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestBucket {
        let bucket = self.labelled_resolve(registrar);
        registrar.consume_bucket(bucket);
        bucket
    }
}

//=====================
// BUCKET BATCHES
//=====================

pub trait ConsumedBucketBatch {
    fn resolve_and_consume(self, registrar: &ManifestNameRegistrar) -> ManifestBucketBatch;
}

impl<B: LabelledResolve<ManifestBucketBatch>> ConsumedBucketBatch for B {
    fn resolve_and_consume(self, registrar: &ManifestNameRegistrar) -> ManifestBucketBatch {
        let bucket_batch = self.labelled_resolve(registrar);
        match &bucket_batch {
            ManifestBucketBatch::ManifestBuckets(owned_buckets) => {
                for owned_bucket in owned_buckets {
                    registrar.consume_bucket(*owned_bucket);
                }
            }
            ManifestBucketBatch::EntireWorktop => {
                // No named buckets are consumed - instead EntireWorktop refers only to the
                // unnamed buckets on the worktop part of the transaction processor
            }
        }
        bucket_batch
    }
}

//=====================
// PROOFS
//=====================

impl LabelResolver<ManifestProof> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> ManifestProof {
        self.name_lookup().proof(name)
    }
}

/// Represents a new [`ManifestProof`] which needs registering.
#[must_use]
pub struct NamedManifestProof {
    namer_id: ManifestNamerId,
    name: String,
}

labelled_resolvable_with_identity_impl!(NamedManifestProof, resolver_output: Self);

impl LabelResolver<NamedManifestProof> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> NamedManifestProof {
        self.new_proof(name)
    }
}

/// Binds a name for a new [`ManifestProof`].
pub trait NewManifestProof {
    fn register(self, registrar: &ManifestNameRegistrar);
}

impl<T: LabelledResolve<NamedManifestProof>> NewManifestProof for T {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_proof(self.labelled_resolve(registrar));
    }
}

/// An existing proof. Its handle will NOT be consumed by this instruction.
///
/// Accepts a string referencing the name of an existing created proof,
/// or an existing proof from a [`ManifestNameLookup`].
pub trait ReferencedManifestProof {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestProof;
}

impl<T: LabelledResolve<ManifestProof>> ReferencedManifestProof for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestProof {
        let resolved = self.labelled_resolve(registrar);
        registrar.core.borrow().assert_proof_exists(resolved);
        resolved
    }
}

/// A proof which whose name/handle will be marked as used by this action.
///
/// This doesn't necessarily mean that the underlying proof will be dropped,
/// but if its name/handle in the manifest can no longer be used.
///
/// Accepts a string referencing the name of an existing created proof,
/// or an existing proof from a [`ManifestNameLookup`].
pub trait ConsumedManifestProof {
    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestProof;
}

impl<T: LabelledResolve<ManifestProof>> ConsumedManifestProof for T {
    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestProof {
        let proof = self.labelled_resolve(registrar);
        registrar.consume_proof(proof);
        proof
    }
}

//=====================
// INTENTS
//=====================

impl LabelResolver<ManifestNamedIntent> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> ManifestNamedIntent {
        self.name_lookup().intent(name)
    }
}

/// Represents a new [`ManifestNamedIntent`] which needs registering.
#[must_use]
pub struct NamedManifestIntent {
    namer_id: ManifestNamerId,
    name: String,
}

labelled_resolvable_with_identity_impl!(NamedManifestIntent, resolver_output: Self);

impl LabelResolver<NamedManifestIntent> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> NamedManifestIntent {
        self.new_intent(name)
    }
}

/// Binds a name for a new manifest intent.
pub trait NewManifestIntent {
    fn register(self, registrar: &ManifestNameRegistrar);
}

impl<T: LabelledResolve<NamedManifestIntent>> NewManifestIntent for T {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_intent(self.labelled_resolve(registrar));
    }
}

/// An existing manifest intent. Its handle will NOT be consumed by this instruction.
///
/// Accepts a string referencing the name of an existing created named intent
/// (created with `USE_CHILD`), or an existing intent from a [`ManifestNameLookup`].
pub trait ReferencedManifestIntent {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestNamedIntent;
}

impl<T: LabelledResolve<ManifestNamedIntent>> ReferencedManifestIntent for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestNamedIntent {
        let resolved = self.labelled_resolve(registrar);
        registrar.core.borrow().assert_intent_exists(resolved);
        resolved
    }
}

//=====================
// ADDRESS RESERVATIONS
//=====================

impl LabelResolver<ManifestAddressReservation> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> ManifestAddressReservation {
        self.name_lookup().address_reservation(name)
    }
}

/// Represents a new [`ManifestAddressReservation`] which needs registering.
#[must_use]
pub struct NamedManifestAddressReservation {
    namer_id: ManifestNamerId,
    name: String,
}

labelled_resolvable_with_identity_impl!(NamedManifestAddressReservation, resolver_output: Self);

impl LabelResolver<NamedManifestAddressReservation> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> NamedManifestAddressReservation {
        self.new_address_reservation(name)
    }
}

/// Binds a name for a new [`ManifestAddressReservation`].
///
/// Accepts a string representing the name to use for the address reservation,
/// or a newly created address reservation from a [`ManifestNameRegistrar`].
pub trait NewManifestAddressReservation: Sized {
    fn into_named(self, registrar: &ManifestNameRegistrar) -> NamedManifestAddressReservation;

    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_address_reservation(self.into_named(registrar))
    }

    fn register_and_yield(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        let named = self.into_named(registrar);
        let name = named.name.clone();
        registrar.register_address_reservation(named);
        registrar.name_lookup().address_reservation(name)
    }
}

impl<T: LabelledResolve<NamedManifestAddressReservation>> NewManifestAddressReservation for T {
    fn into_named(self, registrar: &ManifestNameRegistrar) -> NamedManifestAddressReservation {
        self.labelled_resolve(registrar)
    }
}

/// An address reservation handle will NOT be consumed by this instruction.
pub trait ReferencedManifestAddressReservation {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation;
}

impl<T: LabelledResolve<ManifestAddressReservation>> ReferencedManifestAddressReservation for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        let address_reservation = self.labelled_resolve(registrar);
        registrar
            .core
            .borrow()
            .assert_address_reservation_exists(address_reservation);
        address_reservation
    }
}

/// An address reservation whose name/handle will be marked as used by this action.
///
/// Accepts a string referencing the name of an existing created address reservation,
/// or an existing address reservation from a [`ManifestNameLookup`].
pub trait ConsumedManifestAddressReservation {
    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation;
}

impl<T: LabelledResolve<ManifestAddressReservation>> ConsumedManifestAddressReservation for T {
    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        let address_reservation = self.labelled_resolve(registrar);
        registrar.consume_address_reservation(address_reservation);
        address_reservation
    }
}

pub trait ConsumedOptionalManifestAddressReservation {
    fn mark_consumed(self, registrar: &ManifestNameRegistrar)
        -> Option<ManifestAddressReservation>;
}

impl<T: LabelledResolve<Option<ManifestAddressReservation>>>
    ConsumedOptionalManifestAddressReservation for T
{
    fn mark_consumed(
        self,
        registrar: &ManifestNameRegistrar,
    ) -> Option<ManifestAddressReservation> {
        let reservation = self.labelled_resolve(registrar);
        if let Some(reservation) = reservation {
            registrar.consume_address_reservation(reservation);
        }
        reservation
    }
}

//=====================
// NAMED ADDRESSES
//=====================

impl LabelResolver<ManifestNamedAddress> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> ManifestNamedAddress {
        self.name_lookup().named_address(name)
    }
}
/// Represents a new [`ManifestNamedAddress`] which needs registering.
#[must_use]
pub struct NamedManifestAddress {
    namer_id: ManifestNamerId,
    name: String,
}

labelled_resolvable_with_identity_impl!(NamedManifestAddress, resolver_output: Self);

impl LabelResolver<NamedManifestAddress> for ManifestNameRegistrar {
    fn resolve_label_into(&self, name: &str) -> NamedManifestAddress {
        self.new_named_address(name)
    }
}

/// Binds a name for a new [`ManifestAddressReservation`].
///
/// Accepts a string representing the name to use for the address,
/// or a newly created address from a [`ManifestNameRegistrar`].
pub trait NewNamedManifestAddress {
    fn register(self, registrar: &ManifestNameRegistrar);
}

impl<T: LabelledResolve<NamedManifestAddress>> NewNamedManifestAddress for T {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_named_address(self.labelled_resolve(registrar));
    }
}

/// An address handle will NOT be consumed by this instruction.
pub trait ReferencedManifestGlobalAddress {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestGlobalAddress;
}

impl<T: LabelledResolve<ManifestGlobalAddress>> ReferencedManifestGlobalAddress for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestGlobalAddress {
        let address = self.labelled_resolve(registrar);
        if let ManifestGlobalAddress::Named(named_address) = address {
            registrar
                .core
                .borrow()
                .assert_named_address_exists(named_address);
        }
        address
    }
}

/// An address handle will NOT be consumed by this instruction.
pub trait ReferencedManifestComponentAddress {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestComponentAddress;
}

impl<T: LabelledResolve<ManifestComponentAddress>> ReferencedManifestComponentAddress for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestComponentAddress {
        let address = self.labelled_resolve(registrar);
        if let ManifestComponentAddress::Named(named_address) = address {
            registrar
                .core
                .borrow()
                .assert_named_address_exists(named_address);
        }
        address
    }
}

/// An address handle will NOT be consumed by this instruction.
pub trait ReferencedManifestResourceAddress {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestResourceAddress;
}

impl<T: LabelledResolve<ManifestResourceAddress>> ReferencedManifestResourceAddress for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestResourceAddress {
        let address = self.labelled_resolve(registrar);
        if let ManifestResourceAddress::Named(named_address) = address {
            registrar
                .core
                .borrow()
                .assert_named_address_exists(named_address);
        }
        address
    }
}

/// An address handle will NOT be consumed by this instruction.
pub trait ReferencedManifestPackageAddress {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestPackageAddress;
}

impl<T: LabelledResolve<ManifestPackageAddress>> ReferencedManifestPackageAddress for T {
    fn resolve_referenced(self, registrar: &ManifestNameRegistrar) -> ManifestPackageAddress {
        let address = self.labelled_resolve(registrar);
        if let ManifestPackageAddress::Named(named_address) = address {
            registrar
                .core
                .borrow()
                .assert_named_address_exists(named_address);
        }
        address
    }
}

//=====================
// ARGUMENTS
//=====================

pub trait ResolvableArguments {
    fn resolve(self) -> ManifestValue;
}

impl<T: ManifestEncode + ManifestSborTuple> ResolvableArguments for T {
    fn resolve(self) -> ManifestValue {
        manifest_decode(&manifest_encode(&self).unwrap()).unwrap()
    }
}
