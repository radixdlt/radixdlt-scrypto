use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::KernelNodeApi;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ResourceOperationError(ResourceOperationError),
    ProofError(ProofError),
    CouldNotCreateProof,
}

/// A transient resource container.
#[derive(Debug)]
pub struct BucketSubstate {
    resource: Rc<RefCell<LockableResource>>,
}

impl BucketSubstate {
    pub fn new(resource: Resource) -> Self {
        Self {
            resource: Rc::new(RefCell::new(resource.into())),
        }
    }

    pub fn put(&mut self, other: BucketSubstate) -> Result<(), ResourceOperationError> {
        self.borrow_resource_mut().put(other.resource()?)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Resource, ResourceOperationError> {
        self.borrow_resource_mut().take_by_amount(amount)
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<Resource, ResourceOperationError> {
        self.borrow_resource_mut().take_by_ids(ids)
    }

    pub fn create_proof(&mut self, self_bucket_id: BucketId) -> Result<ProofSubstate, ProofError> {
        let container_id = ResourceContainerId::Bucket(self_bucket_id);
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible { .. } => self.create_proof_by_ids(
                &self
                    .total_ids()
                    .expect("Failed to list non-fungible IDs on non-fungible Bucket"),
                container_id,
            ),
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        container_id: ResourceContainerId,
    ) -> Result<ProofSubstate, ProofError> {
        // lock the specified amount
        let locked_amount_or_ids = self
            .borrow_resource_mut()
            .lock_by_amount(amount)
            .map_err(ProofError::ResourceOperationError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.resource.clone(), locked_amount_or_ids.clone()),
        );
        ProofSubstate::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
        container_id: ResourceContainerId,
    ) -> Result<ProofSubstate, ProofError> {
        // lock the specified id set
        let locked_amount_or_ids = self
            .borrow_resource_mut()
            .lock_by_ids(ids)
            .map_err(ProofError::ResourceOperationError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.resource.clone(), locked_amount_or_ids.clone()),
        );
        ProofSubstate::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.borrow_resource().resource_address()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.borrow_resource().resource_type()
    }

    pub fn total_amount(&self) -> Decimal {
        self.borrow_resource().total_amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, ResourceOperationError> {
        self.borrow_resource().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_resource().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_resource().is_empty()
    }

    pub fn resource(self) -> Result<Resource, ResourceOperationError> {
        Rc::try_unwrap(self.resource)
            .map_err(|_| ResourceOperationError::ResourceLocked)
            .map(|c| c.into_inner())
            .map(Into::into)
    }

    pub fn borrow_resource(&self) -> Ref<LockableResource> {
        self.resource.borrow()
    }

    pub fn borrow_resource_mut(&mut self) -> RefMut<LockableResource> {
        self.resource.borrow_mut()
    }

    pub fn peek_resource(&self) -> Resource {
        let lockable_resource: &LockableResource = &self.borrow_resource();
        lockable_resource.peek_resource()
    }
}

pub struct BucketBlueprint;

impl BucketBlueprint {
    pub(crate) fn take<Y>(
        receiver: BucketId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: BucketTakeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket_handle = api.lock_substate(
            RENodeId::Bucket(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket.take(input.amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub(crate) fn take_non_fungibles<Y>(
        receiver: BucketId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: BucketTakeNonFungiblesInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket_handle = api.lock_substate(
            RENodeId::Bucket(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket.take_non_fungibles(&input.ids).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Bucket(bucket_id)))
    }

    pub(crate) fn put<Y>(
        receiver: BucketId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: BucketPutInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket_handle = api.lock_substate(
            RENodeId::Bucket(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let other_bucket = api.drop_node(RENodeId::Bucket(input.bucket.0))?.into();
        let mut substate_mut = api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        bucket.put(other_bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn create_proof<Y>(
        receiver: BucketId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: BucketCreateProofInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket_handle = api.lock_substate(
            RENodeId::Bucket(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let proof = bucket.create_proof(receiver).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?;

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
        receiver: BucketId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: BucketGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket_handle = api.lock_substate(
            RENodeId::Bucket(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.get_ref(bucket_handle)?;
        let bucket = substate_ref.bucket();
        let ids = bucket.total_ids().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        Ok(IndexedScryptoValue::from_typed(&ids))
    }

    pub(crate) fn get_amount<Y>(
        receiver: BucketId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: BucketGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket_handle = api.lock_substate(
            RENodeId::Bucket(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::read_only(),
        )?;

        let substate = api.get_ref(bucket_handle)?;
        let bucket = substate.bucket();
        Ok(IndexedScryptoValue::from_typed(&bucket.total_amount()))
    }

    pub(crate) fn get_resource_address<Y>(
        receiver: BucketId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: BucketGetResourceAddressInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let bucket_handle = api.lock_substate(
            RENodeId::Bucket(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Bucket(BucketOffset::Bucket),
            LockFlags::read_only(),
        )?;

        let substate = api.get_ref(bucket_handle)?;
        let bucket = substate.bucket();

        Ok(IndexedScryptoValue::from_typed(&bucket.resource_address()))
    }
}
