use crate::{internal_prelude::*, manifest::decompiler::ManifestObjectNames};

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
    object_names: ManifestObjectNames,
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

    pub fn named_address_id(&self, name: impl AsRef<str>) -> u32 {
        match self.core.borrow().resolve_named_address(name) {
            ManifestAddress::Static(_) => panic!("Named manifest address can't be static"),
            ManifestAddress::Named(id) => id,
        }
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

    pub fn object_names(&self) -> ManifestObjectNames {
        self.core.borrow().object_names.clone()
    }
}

pub enum ManifestObjectState<T> {
    Unregistered,
    Present(T),
    Consumed,
}

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

// NOTE:
//------
// Addresses are more complicated than buckets/proofs - eg:
// * They can be static or named
// * We want to provide tighter bounds (eg distinguish Resource / Package / Global)
// * Addresses and Address Reservations are both used together and could be confused
//
// So we purposefully don't support the New_/Existing_ traits for named addresses or
// address reservations.
//
// Instead, users have to use an explicit namer

#[must_use]
pub struct NamedManifestAddressReservation {
    namer_id: ManifestNamerId,
    name: String,
}

#[must_use]
pub struct NamedManifestAddress {
    namer_id: ManifestNamerId,
    name: String,
}

//=================================================
// OTHER RESOLVABLE TRAITS FOR THE MANIFEST BUILDER
//=================================================

pub trait ResolvableComponentAddress {
    fn resolve(self, registrar: &ManifestNameRegistrar) -> DynamicComponentAddress;
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

pub trait ResolvableDecimal {
    fn resolve(self) -> Decimal;
}

impl<A: TryInto<Decimal, Error = E>, E: Debug> ResolvableDecimal for A {
    fn resolve(self) -> Decimal {
        self.try_into().expect("Decimal was not valid")
    }
}

pub trait ResolvableArguments {
    fn resolve(self) -> ManifestValue;
}

impl<T: ManifestEncode + ManifestSborTuple> ResolvableArguments for T {
    fn resolve(self) -> ManifestValue {
        manifest_decode(&manifest_encode(&self).unwrap()).unwrap()
    }
}
