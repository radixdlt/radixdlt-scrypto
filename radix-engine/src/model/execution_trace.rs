use sbor::rust::collections::HashMap;
use scrypto::engine::types::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceChange {
    pub resource_address: ResourceAddress,
    pub component_address: ComponentAddress,
    pub vault_id: VaultId,
    pub amount: Decimal,
}

pub struct ExecutionTraceReceipt {
    pub resource_changes: Vec<ResourceChange>,
}

#[derive(Debug)]
pub struct ExecutionTrace {
    pub resource_changes: HashMap<ComponentAddress, HashMap<VaultId, (ResourceAddress, Decimal)>>,
}

impl ExecutionTrace {
    pub fn new() -> ExecutionTrace {
        Self {
            resource_changes: HashMap::new(),
        }
    }

    pub fn to_receipt(self) -> ExecutionTraceReceipt {
        let resource_changes: Vec<ResourceChange> = self
            .resource_changes
            .into_iter()
            .flat_map(|(component_address, v)| {
                v.into_iter().map(
                    move |(vault_id, (resource_address, amount))| ResourceChange {
                        resource_address,
                        component_address,
                        vault_id,
                        amount,
                    },
                )
            })
            .filter(|el| !el.amount.is_zero())
            .collect();
        ExecutionTraceReceipt { resource_changes }
    }
}
