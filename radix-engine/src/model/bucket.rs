use sbor::*;
use scrypto::buffer::scrypto_decode;
use crate::engine::SystemApi;
use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::values::ScryptoValue;

use crate::model::{
    Proof, ProofError, ResourceContainer, ResourceContainerError, ResourceContainerId,
};

#[derive(Debug, Clone, PartialEq)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    MethodNotFound(String),
    ResourceContainerError(ResourceContainerError),
    ProofError(ProofError),
    CouldNotCreateProof,
}

/// A transient resource container.
#[derive(Debug)]
pub struct Bucket {
    container: Rc<RefCell<ResourceContainer>>,
}

impl Bucket {
    pub fn new(container: ResourceContainer) -> Self {
        Self {
            container: Rc::new(RefCell::new(container)),
        }
    }

    fn put(&mut self, other: Bucket) -> Result<(), ResourceContainerError> {
        self.borrow_container_mut().put(other.into_container()?)
    }

    fn take(&mut self, amount: Decimal) -> Result<ResourceContainer, ResourceContainerError> {
        self.borrow_container_mut().take_by_amount(amount)
    }

    fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<ResourceContainer, ResourceContainerError> {
        self.borrow_container_mut().take_by_ids(ids)
    }

    pub fn create_proof(&mut self, self_bucket_id: BucketId) -> Result<Proof, ProofError> {
        let container_id = ResourceContainerId::Bucket(self_bucket_id);
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible => {
                self.create_proof_by_ids(&self.total_ids().unwrap(), container_id)
            }
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        container_id: ResourceContainerId,
    ) -> Result<Proof, ProofError> {
        // lock the specified amount
        let locked_amount_or_ids = self
            .borrow_container_mut()
            .lock_by_amount(amount)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.container.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            false,
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        container_id: ResourceContainerId,
    ) -> Result<Proof, ProofError> {
        // lock the specified id set
        let locked_amount_or_ids = self
            .borrow_container_mut()
            .lock_by_ids(ids)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.container.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            false,
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.borrow_container().resource_address()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.borrow_container().resource_type()
    }

    fn total_amount(&self) -> Decimal {
        self.borrow_container().total_amount()
    }

    fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        self.borrow_container().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_container().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_container().is_empty()
    }

    pub fn into_container(self) -> Result<ResourceContainer, ResourceContainerError> {
        Rc::try_unwrap(self.container)
            .map_err(|_| ResourceContainerError::ContainerLocked)
            .map(|c| c.into_inner())
    }

    fn borrow_container(&self) -> Ref<ResourceContainer> {
        self.container.borrow()
    }

    fn borrow_container_mut(&mut self) -> RefMut<ResourceContainer> {
        self.container.borrow_mut()
    }

    pub fn main<S: SystemApi>(
        &mut self,
        bucket_id: BucketId,
        function: &str,
        args: Vec<ScryptoValue>,
        system_api: &mut S,
    ) -> Result<ScryptoValue, BucketError> {
        match function {
            "take_from_bucket" => {
                let amount: Decimal = scrypto_decode(&args[0].raw)
                    .map_err(|e| BucketError::InvalidRequestData(e))?;
                let container = self.take(amount).map_err(BucketError::ResourceContainerError)?;
                let bucket_id = system_api.create_bucket(container).map_err(|_| BucketError::CouldNotCreateBucket)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "take_non_fungibles_from_bucket" => {
                let ids: BTreeSet<NonFungibleId> = scrypto_decode(&args[0].raw)
                    .map_err(|e| BucketError::InvalidRequestData(e))?;
                let container = self.take_non_fungibles(&ids).map_err(BucketError::ResourceContainerError)?;
                let bucket_id = system_api.create_bucket(container).map_err(|_| BucketError::CouldNotCreateBucket)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "get_non_fungible_ids_in_bucket" => {
                let ids = self.total_ids().map_err(BucketError::ResourceContainerError)?;
                Ok(ScryptoValue::from_value(&ids))
            }
            "put_into_bucket" => {
                let bucket_id: scrypto::resource::Bucket = scrypto_decode(&args[0].raw)
                    .map_err(|e| BucketError::InvalidRequestData(e))?;
                let bucket = system_api.take_bucket(bucket_id.0).map_err(|_| BucketError::CouldNotTakeBucket)?;
                self.put(bucket).map_err(BucketError::ResourceContainerError)?;
                Ok(ScryptoValue::from_value(&()))
            }
            "get_bucket_amount" => {
                Ok(ScryptoValue::from_value(&self.total_amount()))
            }
            "get_bucket_resource_address" => {
                Ok(ScryptoValue::from_value(&self.resource_address()))
            }
            "create_bucket_proof" => {
                let proof = self.create_proof(bucket_id).map_err(BucketError::ProofError)?;
                let proof_id = system_api.create_proof(proof).map_err(|_| BucketError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Proof(
                    proof_id
                )))
            }
            _ => Err(BucketError::MethodNotFound(function.to_string())),
        }
    }

    pub fn drop<'s, S: SystemApi>(self, system_api: &mut S) -> Result<ScryptoValue, BucketError> {
        // Notify resource manager, TODO: Should not need to notify manually
        let resource_address = self.resource_address();
        let mut resource_manager = system_api
            .borrow_global_mut_resource_manager(resource_address)
            .unwrap();
        resource_manager.burn(self.total_amount());
        if matches!(resource_manager.resource_type(), ResourceType::NonFungible) {
            for id in self.total_ids().unwrap() {
                let non_fungible_address = NonFungibleAddress::new(resource_address, id);
                system_api.set_non_fungible(non_fungible_address, Option::None);
            }
        }
        system_api.return_borrowed_global_resource_manager(resource_address, resource_manager);

        Ok(ScryptoValue::from_value(&()))
    }
}
