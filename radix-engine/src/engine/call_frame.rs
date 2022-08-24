use transaction::validation::*;

use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::types::*;
use crate::wasm::*;

// TODO: reduce fields visibility

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame {
    /// The frame id
    pub depth: usize,
    /// The running actor of this frame
    pub actor: REActor,

    /// All ref values accessible by this call frame. The value may be located in one of the following:
    /// 1. borrowed values
    /// 2. track
    pub node_refs: HashMap<RENodeId, RENodePointer>,

    /// Owned Values
    pub owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,

    pub auth_zone: AuthZone,
}

impl CallFrame {
    pub fn new_root(signer_public_keys: Vec<EcdsaPublicKey>) -> Self {
        // TODO: Cleanup initialization of authzone
        let signer_non_fungible_ids: BTreeSet<NonFungibleId> = signer_public_keys
            .clone()
            .into_iter()
            .map(|public_key| NonFungibleId::from_bytes(public_key.to_vec()))
            .collect();

        let mut initial_auth_zone_proofs = Vec::new();
        if !signer_non_fungible_ids.is_empty() {
            // Proofs can't be zero amount
            let mut ecdsa_bucket = Bucket::new(ResourceContainer::new_non_fungible(
                ECDSA_TOKEN,
                signer_non_fungible_ids,
            ));
            let ecdsa_proof = ecdsa_bucket
                .create_proof(ECDSA_TOKEN_BUCKET_ID)
                .expect("Failed to construct signature proof");
            initial_auth_zone_proofs.push(ecdsa_proof);
        }

        let auth_zone = AuthZone::new_with_proofs(initial_auth_zone_proofs);

        Self {
            depth: 0,
            actor: REActor {
                // Temporary
                fn_identifier: FnIdentifier::Native(NativeFnIdentifier::TransactionProcessor(
                    TransactionProcessorFnIdentifier::Run,
                )),
                receiver: None,
            },
            node_refs: HashMap::new(),
            owned_heap_nodes: HashMap::new(),
            auth_zone,
        }
    }

    pub fn new_child<'s, W, I, C, Y>(
        depth: usize,
        actor: REActor,
        owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,
        node_refs: HashMap<RENodeId, RENodePointer>,
        _system_api: &mut Y,
    ) -> Self
    where
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
        Y: SystemApi<'s, W, I, C>,
    {
        let auth_zone = AuthZone::new();

        Self {
            depth,
            actor,
            node_refs,
            owned_heap_nodes,
            auth_zone,
        }
    }

    pub fn drop_owned_values(&mut self) -> Result<(), RuntimeError> {
        let values = self
            .owned_heap_nodes
            .drain()
            .map(|(_id, value)| value)
            .collect();
        HeapRENode::drop_nodes(values)
            .map_err(|e| RuntimeError::KernelError(KernelError::DropFailure(e)))
    }

    pub fn take_available_values(
        &mut self,
        node_ids: HashSet<RENodeId>,
        persist_only: bool,
    ) -> Result<(HashMap<RENodeId, HeapRootRENode>, HashSet<RENodeId>), RuntimeError> {
        let (taken, missing) = {
            let mut taken_values = HashMap::new();
            let mut missing_values = HashSet::new();

            for id in node_ids {
                let maybe = self.owned_heap_nodes.remove(&id);
                if let Some(value) = maybe {
                    value.root().verify_can_move()?;
                    if persist_only {
                        value.root().verify_can_persist()?;
                    }
                    taken_values.insert(id, value);
                } else {
                    missing_values.insert(id);
                }
            }

            (taken_values, missing_values)
        };

        // Moved values must have their references removed
        for (id, value) in &taken {
            self.node_refs.remove(id);
            for (id, ..) in &value.child_nodes {
                self.node_refs.remove(id);
            }
        }

        Ok((taken, missing))
    }
}
