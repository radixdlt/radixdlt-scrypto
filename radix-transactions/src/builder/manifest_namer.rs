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
    named_addresses: NonIterMap<String, ManifestObjectState<ManifestAddress>>,
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

    pub fn resolve_named_address(&self, name: impl AsRef<str>) -> ManifestAddress {
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
        let new_address = ManifestAddress::Named(address_id);
        match self.named_addresses.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_address);
                self
                .object_names.address_names.insert(address_id, new.name);
            },
            Some(_) => unreachable!("NewManifestNamedAddress was somehow registered twice"),
            None => unreachable!("NewManifestNamedAddress was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    pub fn check_address_exists(&self, address: impl Into<DynamicGlobalAddress>) {
        if let DynamicGlobalAddress::Named(address_id) = address.into() {
            self.object_names
                .address_names
                .get(&address_id)
                .expect("Address was not recognised - perhaps you're using a named address not sourced from this builder?");
        }
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
            _ => panic!("You cannot reference an intent with name \"{}\" before it has been created with a relevant instruction in the manifest builder, or parent transaction builder", name.as_ref()),
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

    pub fn check_intent_exists(&self, intent: impl Into<ManifestNamedIntent>) {
        self.object_names
            .intent_names
            .get(&intent.into())
            .expect("Intent was not recognised");
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

    pub fn named_address(&self, name: impl AsRef<str>) -> ManifestAddress {
        self.core.borrow().resolve_named_address(name)
    }

    pub fn named_address_id(&self, name: impl AsRef<str>) -> ManifestNamedAddress {
        match self.core.borrow().resolve_named_address(name) {
            ManifestAddress::Static(_) => panic!("Named manifest address can't be static"),
            ManifestAddress::Named(id) => id,
        }
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

    pub fn check_address_exists(&self, address: impl Into<DynamicGlobalAddress>) {
        self.core.borrow().check_address_exists(address)
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
        self.core.borrow().check_intent_exists(intent)
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

#[must_use]
pub struct NamedManifestBucket {
    namer_id: ManifestNamerId,
    name: String,
}

/// Either a string, or a new bucket from a namer.
pub trait NewManifestBucket {
    fn register(self, registrar: &ManifestNameRegistrar);
}

impl<'a> NewManifestBucket for &'a str {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_bucket(registrar.new_bucket(self));
    }
}

impl<'a> NewManifestBucket for &'a String {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_bucket(registrar.new_bucket(self));
    }
}

impl NewManifestBucket for String {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_bucket(registrar.new_bucket(self));
    }
}

impl NewManifestBucket for NamedManifestBucket {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_bucket(self);
    }
}

/// Either a string, or an existing bucket from a namer.
pub trait ExistingManifestBucket: Sized {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestBucket;

    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestBucket {
        let bucket = self.resolve(registrar);
        registrar.consume_bucket(bucket);
        bucket
    }
}

impl<'a> ExistingManifestBucket for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestBucket {
        registrar.name_lookup().bucket(self)
    }
}

impl<'a> ExistingManifestBucket for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestBucket {
        registrar.name_lookup().bucket(self)
    }
}

impl ExistingManifestBucket for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestBucket {
        registrar.name_lookup().bucket(self)
    }
}

impl ExistingManifestBucket for ManifestBucket {
    fn resolve(self, _registrar: &ManifestNameRegistrar) -> ManifestBucket {
        self
    }
}

//=====================
// BUCKET BATCHES
//=====================

pub trait ResolvableBucketBatch {
    fn consume_and_resolve(self, registrar: &ManifestNameRegistrar) -> ManifestValue;
}

impl<B: ExistingManifestBucket> ResolvableBucketBatch for BTreeSet<B> {
    fn consume_and_resolve(self, registrar: &ManifestNameRegistrar) -> ManifestValue {
        let buckets: Vec<_> = self
            .into_iter()
            .map(|b| b.mark_consumed(registrar))
            .collect();
        manifest_decode(&manifest_encode(&buckets).unwrap()).unwrap()
    }
}

impl<B: ExistingManifestBucket, const N: usize> ResolvableBucketBatch for [B; N] {
    fn consume_and_resolve(self, registrar: &ManifestNameRegistrar) -> ManifestValue {
        let buckets: Vec<_> = self
            .into_iter()
            .map(|b| b.mark_consumed(registrar))
            .collect();
        manifest_decode(&manifest_encode(&buckets).unwrap()).unwrap()
    }
}

impl<B: ExistingManifestBucket> ResolvableBucketBatch for Vec<B> {
    fn consume_and_resolve(self, registrar: &ManifestNameRegistrar) -> ManifestValue {
        let buckets: Vec<_> = self
            .into_iter()
            .map(|b| b.mark_consumed(registrar))
            .collect();
        manifest_decode(&manifest_encode(&buckets).unwrap()).unwrap()
    }
}

impl ResolvableBucketBatch for ManifestExpression {
    fn consume_and_resolve(self, _: &ManifestNameRegistrar) -> ManifestValue {
        match &self {
            ManifestExpression::EntireWorktop => {
                // No named buckets are consumed - instead EntireWorktop refers only to the
                // unnamed buckets on the worktop part of the transaction processor
                manifest_decode(&manifest_encode(&self).unwrap()).unwrap()
            }
            ManifestExpression::EntireAuthZone => {
                panic!("Not an allowed expression for a batch of buckets")
            }
        }
    }
}

//=====================
// PROOFS
//=====================

#[must_use]
pub struct NamedManifestProof {
    namer_id: ManifestNamerId,
    name: String,
}

/// Either a string, or a new proof from a namer.
pub trait NewManifestProof {
    fn register(self, registrar: &ManifestNameRegistrar);
}

impl<'a> NewManifestProof for &'a str {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_proof(registrar.new_proof(self));
    }
}

impl<'a> NewManifestProof for &'a String {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_proof(registrar.new_proof(self));
    }
}

impl NewManifestProof for String {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_proof(registrar.new_proof(self));
    }
}

impl NewManifestProof for NamedManifestProof {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_proof(self);
    }
}

/// Either a string, or an existing proof from a namer.
pub trait ExistingManifestProof: Sized {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestProof;

    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestProof {
        let proof = self.resolve(registrar);
        registrar.consume_proof(proof);
        proof
    }
}

impl<'a> ExistingManifestProof for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestProof {
        registrar.name_lookup().proof(self)
    }
}

impl<'a> ExistingManifestProof for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestProof {
        registrar.name_lookup().proof(self)
    }
}

impl ExistingManifestProof for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestProof {
        registrar.name_lookup().proof(self)
    }
}

impl ExistingManifestProof for ManifestProof {
    fn resolve(self, _registrar: &ManifestNameRegistrar) -> ManifestProof {
        self
    }
}

//=====================
// INTENTS
//=====================

#[must_use]
pub struct NamedManifestIntent {
    namer_id: ManifestNamerId,
    name: String,
}

/// Either a string, or a new intent from a namer.
pub trait NewManifestIntent {
    fn register(self, registrar: &ManifestNameRegistrar);
}

impl<'a> NewManifestIntent for &'a str {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_intent(registrar.new_intent(self));
    }
}

impl<'a> NewManifestIntent for &'a String {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_intent(registrar.new_intent(self));
    }
}

impl NewManifestIntent for String {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_intent(registrar.new_intent(self));
    }
}

impl NewManifestIntent for NamedManifestIntent {
    fn register(self, registrar: &ManifestNameRegistrar) {
        registrar.register_intent(self);
    }
}

pub trait ExistingManifestIntent: Sized {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestNamedIntent;
}

impl<'a> ExistingManifestIntent for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestNamedIntent {
        registrar.name_lookup().intent(self)
    }
}

impl<'a> ExistingManifestIntent for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestNamedIntent {
        registrar.name_lookup().intent(self)
    }
}

impl ExistingManifestIntent for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestNamedIntent {
        registrar.name_lookup().intent(self)
    }
}

impl ExistingManifestIntent for ManifestNamedIntent {
    fn resolve(self, _registrar: &ManifestNameRegistrar) -> ManifestNamedIntent {
        self
    }
}

//=====================
// ADDRESS RESERVATIONS
//=====================

#[must_use]
pub struct NamedManifestAddressReservation {
    namer_id: ManifestNamerId,
    name: String,
}

/// Either a string, or a new manifest address reservation from a namer.
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

impl<'a> NewManifestAddressReservation for &'a str {
    fn into_named(self, registrar: &ManifestNameRegistrar) -> NamedManifestAddressReservation {
        registrar.new_address_reservation(self)
    }
}

impl<'a> NewManifestAddressReservation for &'a String {
    fn into_named(self, registrar: &ManifestNameRegistrar) -> NamedManifestAddressReservation {
        registrar.new_address_reservation(self)
    }
}

impl NewManifestAddressReservation for String {
    fn into_named(self, registrar: &ManifestNameRegistrar) -> NamedManifestAddressReservation {
        registrar.new_address_reservation(self)
    }
}

impl NewManifestAddressReservation for NamedManifestAddressReservation {
    fn into_named(self, _registrar: &ManifestNameRegistrar) -> NamedManifestAddressReservation {
        self
    }
}

pub trait OptionalExistingManifestAddressReservation: Sized {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> Option<ManifestAddressReservation>;

    fn mark_consumed(
        self,
        registrar: &ManifestNameRegistrar,
    ) -> Option<ManifestAddressReservation> {
        let reservation = self.resolve(registrar);
        if let Some(reservation) = reservation {
            registrar.consume_address_reservation(reservation);
        }
        reservation
    }
}

impl<T: ExistingManifestAddressReservation> OptionalExistingManifestAddressReservation for T {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> Option<ManifestAddressReservation> {
        Some(<Self as ExistingManifestAddressReservation>::resolve(
            self, registrar,
        ))
    }
}

// We only implement it for one Option, so that `None` has a unique implementation
// We choose Option<String> for backwards compatibility
impl OptionalExistingManifestAddressReservation for Option<String> {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> Option<ManifestAddressReservation> {
        self.map(|r| <String as ExistingManifestAddressReservation>::resolve(r, registrar))
    }
}

pub trait ExistingManifestAddressReservation: Sized {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation;

    fn mark_consumed(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        let reservation = self.resolve(registrar);
        registrar.consume_address_reservation(reservation);
        reservation
    }
}

impl<'a> ExistingManifestAddressReservation for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        registrar.name_lookup().address_reservation(self)
    }
}

impl<'a> ExistingManifestAddressReservation for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        registrar.name_lookup().address_reservation(self)
    }
}

impl<'a> ExistingManifestAddressReservation for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        registrar.name_lookup().address_reservation(self)
    }
}

impl<'a> ExistingManifestAddressReservation for ManifestAddressReservation {
    fn resolve(self, _registrar: &ManifestNameRegistrar) -> ManifestAddressReservation {
        self
    }
}

//=====================
// NAMED ADDRESSES
//=====================

// Unlike the above, addresses are a bit more complicated -- so we have traits
// like ResolvablePackageAddress which can be used instead of an
// ExistingNamedManifestAddress trait.

#[must_use]
pub struct NamedManifestAddress {
    namer_id: ManifestNamerId,
    name: String,
}

pub trait ResolvableComponentAddress {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicComponentAddress;
}

impl<'a> ResolvableComponentAddress for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicComponentAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvableComponentAddress for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicComponentAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvableComponentAddress for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicComponentAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug> ResolvableComponentAddress for A {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicComponentAddress {
        let address = self
            .try_into()
            .expect("Address was not valid ComponentAddress");
        registrar.check_address_exists(address);
        address
    }
}

pub trait ResolvableResourceAddress: Sized {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicResourceAddress;

    /// Note - this can be removed when all the static resource addresses in the
    /// manifest instructions are gone
    fn resolve_static(self, registrar: &ManifestNameRegistrar) -> ResourceAddress {
        match self.resolve(registrar) {
            DynamicResourceAddress::Static(address) => address,
            DynamicResourceAddress::Named(_) => {
                panic!("This address needs to be a static/fixed address")
            }
        }
    }
}

impl<A: TryInto<DynamicResourceAddress, Error = E>, E: Debug> ResolvableResourceAddress for A {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicResourceAddress {
        let address = self
            .try_into()
            .expect("Address was not valid ResourceAddress");
        registrar.check_address_exists(address);
        address
    }
}

impl<'a> ResolvableResourceAddress for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicResourceAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvableResourceAddress for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicResourceAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvableResourceAddress for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicResourceAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

pub trait ResolvablePackageAddress: Sized {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicPackageAddress;

    /// Note - this can be removed when all the static package addresses in the
    /// manifest instructions are gone
    fn resolve_static(self, registrar: &ManifestNameRegistrar) -> PackageAddress {
        match self.resolve(registrar) {
            DynamicPackageAddress::Static(address) => address,
            DynamicPackageAddress::Named(_) => {
                panic!("This address needs to be a static/fixed address")
            }
        }
    }
}

impl<A: TryInto<DynamicPackageAddress, Error = E>, E: Debug> ResolvablePackageAddress for A {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicPackageAddress {
        let address = self
            .try_into()
            .expect("Address was not valid PackageAddress");
        registrar.check_address_exists(address);
        address
    }
}

impl<'a> ResolvablePackageAddress for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicPackageAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvablePackageAddress for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicPackageAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvablePackageAddress for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicPackageAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

pub trait ResolvableGlobalAddress {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicGlobalAddress;
}

impl<A: TryInto<DynamicGlobalAddress, Error = E>, E: Debug> ResolvableGlobalAddress for A {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicGlobalAddress {
        let address = self
            .try_into()
            .expect("Address was not valid GlobalAddress");
        registrar.check_address_exists(address);
        address
    }
}

impl<'a> ResolvableGlobalAddress for &'a str {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicGlobalAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvableGlobalAddress for &'a String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicGlobalAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

impl<'a> ResolvableGlobalAddress for String {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicGlobalAddress {
        registrar.name_lookup().named_address_id(self).into()
    }
}

//=====================
// DECIMAL
//=====================

pub trait ResolvableDecimal {
    fn resolve(self) -> Decimal;
}

impl<A: TryInto<Decimal, Error = E>, E: Debug> ResolvableDecimal for A {
    fn resolve(self) -> Decimal {
        self.try_into().expect("Decimal was not valid")
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
