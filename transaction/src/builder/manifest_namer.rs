use crate::{internal_prelude::*, manifest::decompiler::ManifestObjectNames};

pub struct ManifestNamer {
    core: Rc<RefCell<ManifestNamerCore>>,
}

pub struct ManifestNameRegistrar {
    core: Rc<RefCell<ManifestNamerCore>>,
}

static GLOBAL_INCREMENTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

#[derive(PartialEq, Eq, Clone, Copy, Default)]
struct ManifestNamerId(u64);

impl ManifestNamerId {
    pub fn new_unique() -> Self {
        Self(GLOBAL_INCREMENTER.fetch_add(1, core::sync::atomic::Ordering::Acquire))
    }
}

#[derive(Default)]
struct ManifestNamerCore {
    /// The ManifestNamerId is a mechanism to try to avoid people accidentally mismatching namers and builders
    /// Such a mismatch would create unexpected behaviour
    namer_id: ManifestNamerId,
    id_allocator: ManifestIdAllocator,
    named_buckets: IndexMap<String, ManifestObjectState<ManifestBucket>>,
    named_proofs: IndexMap<String, ManifestObjectState<ManifestProof>>,
    named_addresses: NonIterMap<String, ManifestObjectState<ManifestAddress>>,
    named_address_reservations: NonIterMap<String, ManifestObjectState<ManifestAddressReservation>>,
    object_names: ManifestObjectNames,
}

impl ManifestNamer {
    pub fn new() -> (Self, ManifestNameRegistrar) {
        let core = Rc::new(RefCell::new(ManifestNamerCore {
            namer_id: ManifestNamerId::new_unique(),
            ..Default::default()
        }));
        let namer = Self { core: core.clone() };
        let registrar = ManifestNameRegistrar { core };
        (namer, registrar)
    }

    pub fn new_bucket(&mut self, name: impl Into<String>) -> NewManifestBucket {
        let mut core = self.core.borrow_mut();
        let name = name.into();
        let old_entry = core
            .named_buckets
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new bucket with the same name \"{name}\" multiple times");
        }
        NewManifestBucket {
            namer_id: core.namer_id,
            name,
        }
    }

    pub fn bucket(&mut self, name: impl AsRef<str>) -> ManifestBucket {
        match self.core.borrow().named_buckets.get(name.as_ref()) {
            Some(ManifestObjectState::Present(bucket)) => bucket.clone(),
            Some(ManifestObjectState::Consumed) => panic!("Bucket with name \"{}\" has already been consumed", name.as_ref()),
            _ => panic!("You cannot use a bucket with name \"{}\" before creating it with `new_bucket` and passing that into the relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    pub fn new_proof(&mut self, name: impl Into<String>) -> NewManifestProof {
        let mut core = self.core.borrow_mut();
        let name = name.into();
        let old_entry = core
            .named_proofs
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new proof with the same name \"{name}\" multiple times");
        }
        NewManifestProof {
            namer_id: core.namer_id,
            name,
        }
    }

    pub fn proof(&mut self, name: impl AsRef<str>) -> ManifestProof {
        match self.core.borrow().named_proofs.get(name.as_ref()) {
            Some(ManifestObjectState::Present(proof)) => proof.clone(),
            Some(ManifestObjectState::Consumed) => panic!("Proof with name \"{}\" has already been consumed", name.as_ref()),
            _ => panic!("You cannot use a proof with name \"{}\" before creating it with `new_proof` and passing that into the relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    pub fn new_address_reservation(
        &mut self,
        name: impl Into<String>,
    ) -> NewManifestAddressReservation {
        let mut core = self.core.borrow_mut();
        let name = name.into();
        let old_entry = core
            .named_address_reservations
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new address reservation with the same name \"{name}\" multiple times");
        }
        NewManifestAddressReservation {
            namer_id: core.namer_id,
            name,
        }
    }

    pub fn address_reservation(&mut self, name: impl AsRef<str>) -> ManifestAddressReservation {
        match self.core.borrow().named_address_reservations.get(name.as_ref()) {
            Some(ManifestObjectState::Present(address_reservation)) => address_reservation.clone(),
            Some(ManifestObjectState::Consumed) => panic!("Address reservation with name \"{}\" has already been consumed", name.as_ref()),
            _ => panic!("You cannot use an address reservation with name \"{}\" before creating it with `new_address_reservation` and passing that into the relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    pub fn new_named_address(&mut self, name: impl Into<String>) -> NewManifestNamedAddress {
        let mut core = self.core.borrow_mut();
        let name = name.into();
        let old_entry = core
            .named_addresses
            .insert(name.clone(), ManifestObjectState::Unregistered);
        if old_entry.is_some() {
            panic!("You cannot create a new named address with the same name \"{name}\" multiple times");
        }
        NewManifestNamedAddress {
            namer_id: core.namer_id,
            name,
        }
    }

    pub fn named_address(&mut self, name: impl AsRef<str>) -> ManifestAddress {
        match self.core.borrow().named_addresses.get(name.as_ref()) {
            Some(ManifestObjectState::Present(address)) => address.clone(),
            Some(ManifestObjectState::Consumed) => unreachable!("Address not consumable"),
            _ => panic!("You cannot use an address with name \"{}\" before creating it with `new_named_address` and passing that into the relevant instruction in the manifest builder", name.as_ref()),
        }
    }

    pub fn object_names(&self) -> ManifestObjectNames {
        self.core.borrow().object_names.clone()
    }
}

impl ManifestNameRegistrar {
    /// This is intended for registering a bucket name to an allocated identifier, as part of processing a manifest
    /// instruction which creates a bucket.
    pub fn register_bucket(&self, new: NewManifestBucket) {
        let mut core = self.core.borrow_mut();
        if core.namer_id != new.namer_id {
            panic!("NewManifestBucket cannot be registered against a different ManifestNamer")
        }
        let new_bucket = core.id_allocator.new_bucket_id();
        match core.named_buckets.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_bucket);
                core
                .object_names.bucket_names.insert(new_bucket, new.name);
            },
            Some(_) => unreachable!("NewManifestBucket was somehow registered twice"),
            None => unreachable!("NewManifestBucket was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    /// Creates a new named bucket pair, using collision avoidance strategy to ensure a name is generated for the bucket.
    /// This is intended for use inside manifest builder helper methods, which create and consume an bucket.
    pub fn new_named_bucket_pair(&self, prefix: &str) -> (NewManifestBucket, ManifestBucket) {
        let mut core = self.core.borrow_mut();
        let namer_id = core.namer_id;
        let bucket = core.id_allocator.new_bucket_id();
        let mut name_counter = 1;
        loop {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            match core.named_buckets.entry(name.to_string()) {
                indexmap::map::Entry::Occupied(_) => {
                    name_counter += 1;
                    continue;
                }
                indexmap::map::Entry::Vacant(entry) => {
                    let new_bucket = NewManifestBucket {
                        namer_id,
                        name: name.clone(),
                    };
                    entry.insert(ManifestObjectState::Present(bucket));
                    core.object_names.bucket_names.insert(bucket, name);
                    return (new_bucket, bucket);
                }
            }
        }
    }

    pub fn consume_bucket(&self, consumed: ManifestBucket) {
        let mut core = self.core.borrow_mut();
        let name = core
            .object_names
            .bucket_names
            .get(&consumed)
            .expect("Consumed bucket was not recognised")
            .to_string();
        let entry = core
            .named_buckets
            .get_mut(&name)
            .expect("Inverse index somehow became inconsistent");
        *entry = ManifestObjectState::Consumed;
    }

    pub fn consume_all_buckets(&self) {
        let mut core = self.core.borrow_mut();
        for (_, state) in core.named_buckets.iter_mut() {
            if let ManifestObjectState::Present(_) = state {
                *state = ManifestObjectState::Consumed;
            }
        }
    }

    /// This is intended for registering a proof name to an allocated identifier, as part of processing a manifest
    /// instruction which creates a proof.
    pub fn register_proof(&self, new: NewManifestProof) {
        let mut core = self.core.borrow_mut();
        if core.namer_id != new.namer_id {
            panic!("NewManifestProof cannot be registered against a different ManifestNamer")
        }
        let new_proof = core.id_allocator.new_proof_id();
        match core.named_proofs.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_proof);
                core
                .object_names.proof_names.insert(new_proof, new.name);
            },
            Some(_) => unreachable!("NewManifestProof was somehow registered twice"),
            None => unreachable!("NewManifestProof was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    /// Creates a new named proof pair, using collision avoidance strategy to ensure a name is generated for the proof.
    /// This is intended for use inside manifest builder helper methods, which create and consume an proof.
    pub fn new_named_proof_pair(&self, prefix: &str) -> (NewManifestProof, ManifestProof) {
        let mut core = self.core.borrow_mut();
        let namer_id = core.namer_id;
        let proof = core.id_allocator.new_proof_id();
        let mut name_counter = 1;
        loop {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            match core.named_proofs.entry(name.to_string()) {
                indexmap::map::Entry::Occupied(_) => {
                    name_counter += 1;
                    continue;
                }
                indexmap::map::Entry::Vacant(entry) => {
                    let new_proof = NewManifestProof {
                        namer_id,
                        name: name.clone(),
                    };
                    entry.insert(ManifestObjectState::Present(proof));
                    core.object_names.proof_names.insert(proof, name);
                    return (new_proof, proof);
                }
            }
        }
    }

    pub fn consume_proof(&self, consumed: ManifestProof) {
        let mut core = self.core.borrow_mut();
        let name = core
            .object_names
            .proof_names
            .get(&consumed)
            .expect("Consumed proof was not recognised")
            .to_string();
        let entry = core
            .named_proofs
            .get_mut(&name)
            .expect("Inverse index somehow became inconsistent");
        *entry = ManifestObjectState::Consumed;
    }

    pub fn consume_all_proofs(&self) {
        let mut core = self.core.borrow_mut();
        for (_, state) in core.named_proofs.iter_mut() {
            if let ManifestObjectState::Present(_) = state {
                *state = ManifestObjectState::Consumed;
            }
        }
    }

    /// This is intended for registering an address reservation to an allocated identifier, as part of processing a manifest
    /// instruction which creates an address reservation.
    pub fn register_address_reservation(&self, new: NewManifestAddressReservation) {
        let mut core = self.core.borrow_mut();
        if core.namer_id != new.namer_id {
            panic!("NewManifestAddressReservation cannot be registered against a different ManifestNamer")
        }
        let new_address_reservation = core.id_allocator.new_address_reservation_id();
        match core.named_address_reservations.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_address_reservation);
                core
                .object_names.address_reservation_names.insert(new_address_reservation, new.name);
            },
            Some(_) => unreachable!("NewManifestAddressReservation was somehow registered twice"),
            None => unreachable!("NewManifestAddressReservation was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    /// Creates a new named address reservation pair, using collision avoidance strategy to ensure a name is generated for the address reservation.
    /// This is intended for use inside manifest builder helper methods, which create and consume an address reservation.
    pub fn new_named_address_reservation_pair(
        &self,
        prefix: &str,
    ) -> (NewManifestAddressReservation, ManifestAddressReservation) {
        let mut core = self.core.borrow_mut();
        let namer_id = core.namer_id;
        let address_reservation = core.id_allocator.new_address_reservation_id();
        let mut name_counter = 1;
        loop {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            match core.named_address_reservations.entry(name.to_string()) {
                non_iter_map::Entry::Occupied(_) => {
                    name_counter += 1;
                    continue;
                }
                non_iter_map::Entry::Vacant(entry) => {
                    let new_address_reservation = NewManifestAddressReservation {
                        namer_id,
                        name: name.clone(),
                    };
                    entry.insert(ManifestObjectState::Present(address_reservation));
                    core.object_names
                        .address_reservation_names
                        .insert(address_reservation, name);
                    return (new_address_reservation, address_reservation);
                }
            }
        }
    }

    pub fn consume_address_reservation(&self, consumed: ManifestAddressReservation) {
        let mut core = self.core.borrow_mut();
        let name = core
            .object_names
            .address_reservation_names
            .get(&consumed)
            .expect("Consumed address reservation was not recognised")
            .to_string();
        let entry = core
            .named_address_reservations
            .get_mut(&name)
            .expect("Inverse index somehow became inconsistent");
        *entry = ManifestObjectState::Consumed;
    }

    /// This is intended for registering an address reservation to an allocated identifier, as part of processing a manifest
    /// instruction which creates a named address.
    pub fn register_named_address(&self, new: NewManifestNamedAddress) {
        let mut core = self.core.borrow_mut();
        if core.namer_id != new.namer_id {
            panic!("NewManifestNamedAddress cannot be registered against a different ManifestNamer")
        }
        let address_id = core.id_allocator.new_address_id();
        let new_address = ManifestAddress::Named(address_id);
        match core.named_addresses.get_mut(&new.name) {
            Some(allocated @ ManifestObjectState::Unregistered) => {
                *allocated = ManifestObjectState::Present(new_address);
                core
                .object_names.address_names.insert(address_id, new.name);
            },
            Some(_) => unreachable!("NewManifestNamedAddress was somehow registered twice"),
            None => unreachable!("NewManifestNamedAddress was somehow created without a corresponding entry being added in the name allocation map"),
        }
    }

    /// Creates a new named address pair, using collision avoidance strategy to ensure a name is generated for the address.
    /// This is intended for use inside manifest builder helper methods, which create and consume an address.
    pub fn new_registered_named_address_pair(
        &self,
        prefix: &str,
    ) -> (NewManifestNamedAddress, ManifestAddress) {
        let mut core = self.core.borrow_mut();
        let namer_id = core.namer_id;
        let address_id = core.id_allocator.new_address_id();
        let named_address = ManifestAddress::Named(address_id);
        let mut name_counter = 1;
        loop {
            let name = if name_counter == 1 {
                prefix.to_string()
            } else {
                format!("{prefix}_{name_counter}")
            };
            match core.named_addresses.entry(name.to_string()) {
                non_iter_map::Entry::Occupied(_) => {
                    name_counter += 1;
                    continue;
                }
                non_iter_map::Entry::Vacant(entry) => {
                    let new_address = NewManifestNamedAddress {
                        namer_id,
                        name: name.clone(),
                    };
                    entry.insert(ManifestObjectState::Present(named_address));
                    core.object_names.address_names.insert(address_id, name);
                    return (new_address, named_address);
                }
            }
        }
    }

    pub fn check_address_exists(&self, address: impl Into<DynamicGlobalAddress>) {
        let core = self.core.borrow();
        if let DynamicGlobalAddress::Named(address_id) = address.into() {
            core.object_names
                .address_names
                .get(&address_id)
                .expect("Address was not recognised");
        }
    }
}

pub enum ManifestObjectState<T> {
    Unregistered,
    Present(T),
    Consumed,
}

#[must_use]
pub struct NewManifestBucket {
    namer_id: ManifestNamerId,
    name: String,
}

#[must_use]
pub struct NewManifestProof {
    namer_id: ManifestNamerId,
    name: String,
}

#[must_use]
pub struct NewManifestAddressReservation {
    namer_id: ManifestNamerId,
    name: String,
}

#[must_use]
pub struct NewManifestNamedAddress {
    namer_id: ManifestNamerId,
    name: String,
}
