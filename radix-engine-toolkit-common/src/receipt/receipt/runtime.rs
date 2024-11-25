use super::base::*;
use super::serializable::SerializableToolkitTransactionReceipt;
use crate::receipt::error::*;
use radix_common::prelude::*;
use radix_engine::blueprints::resource::*;
use radix_engine::object_modules::metadata::*;
use radix_engine::system::system_modules::execution_trace::WorktopChange;
use radix_engine::transaction::*;
use radix_engine_interface::prelude::{MetadataValue, *};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTypeSelector;
impl TypeSelector for RuntimeTypeSelector {
    type Usize = usize;
    type Bytes = Vec<u8>;
    type Decimal = Decimal;

    type NodeId = NodeId;
    type NonFungibleGlobalId = NonFungibleGlobalId;

    type MetadataValue = MetadataValue;

    type WorktopChange = WorktopChange;
}

/// The runtime toolkit transaction receipt.
pub type RuntimeToolkitTransactionReceipt = ToolkitTransactionReceipt<RuntimeTypeSelector>;

impl RuntimeToolkitTransactionReceipt {
    pub fn into_serializable_receipt(
        self,
        address_encoder: &AddressBech32Encoder,
    ) -> Result<SerializableToolkitTransactionReceipt, ToolkitReceiptError> {
        self.contextual_try_into(address_encoder)
    }
}

impl TryFrom<TransactionReceipt> for RuntimeToolkitTransactionReceipt {
    type Error = ToolkitReceiptError;

    fn try_from(value: TransactionReceipt) -> Result<Self, Self::Error> {
        match value {
            TransactionReceipt {
                result:
                    TransactionResult::Commit(CommitResult {
                        outcome: TransactionOutcome::Success(..),
                        state_update_summary,
                        state_updates,
                        execution_trace: Some(execution_trace),
                        application_events,
                        ..
                    }),
                fee_summary,
                ..
            } => Ok(Self::CommitSuccess {
                state_updates_summary: StateUpdatesSummary {
                    new_entities: state_update_summary
                        .new_components
                        .into_iter()
                        .map(|value| value.into_node_id())
                        .chain(
                            state_update_summary
                                .new_resources
                                .into_iter()
                                .map(|value| value.into_node_id()),
                        )
                        .chain(
                            state_update_summary
                                .new_packages
                                .into_iter()
                                .map(|value| value.into_node_id()),
                        )
                        .collect(),
                    // We get the metadata updates from the state updates.
                    metadata_updates: get_metadata_updates(&state_updates),
                    // We get the non-fungible data updates from the state updates directly.
                    non_fungible_data_updates: state_updates
                        .by_node
                        .into_iter()
                        .filter_map(|(node_id, value)| {
                            ResourceAddress::try_from(node_id)
                                .map(|address| (address, value))
                                .ok()
                        })
                        .filter(|(resource_address, _)| !resource_address.is_fungible())
                        .filter_map(
                            |(
                                non_fungible_resource_address,
                                NodeStateUpdates::Delta {
                                    by_partition: updates_by_partition,
                                },
                            )| {
                                let partition_number = MAIN_BASE_PARTITION
                                    .at_offset(
                                        NonFungibleResourceManagerPartitionOffset::DataKeyValue
                                            .as_partition_offset(),
                                    )
                                    .unwrap();

                                updates_by_partition
                                    .get(&partition_number)?
                                    .iter_map_entries()
                                    .filter_map(|(key, value)| match value {
                                        DatabaseUpdateRef::Set(value) => Some((key, value)),
                                        DatabaseUpdateRef::Delete => None,
                                    })
                                    .map(|(key, value)| -> Option<_> {
                                        let key = scrypto_decode::<
                                            NonFungibleResourceManagerDataKeyPayload,
                                        >(key)
                                        .ok()
                                        .map(|id| {
                                            NonFungibleGlobalId::new(
                                                non_fungible_resource_address,
                                                id.content,
                                            )
                                        })?;
                                        let value = scrypto_decode::<
                                            NonFungibleResourceManagerDataEntrySubstate,
                                        >(value)
                                        .ok()
                                        .and_then(|value| value.into_value())
                                        .and_then(|value| scrypto_encode(&value).ok())?;
                                        Some((key, value))
                                    })
                                    .collect::<Option<IndexMap<_, _>>>()
                            },
                        )
                        .flatten()
                        .collect(),
                    // We get the newly minted non-fungibles from the events.
                    newly_minted_non_fungibles: application_events.iter().fold(
                        index_set_new(),
                        |mut acc, (EventTypeIdentifier(emitter, event_name), event_data)| {
                            match emitter {
                                Emitter::Method(node_id, ModuleId::Main) => {
                                    match (ResourceAddress::try_from(*node_id), event_name.as_str())
                                    {
                                        (
                                            Ok(resource_address),
                                            MintNonFungibleResourceEvent::EVENT_NAME,
                                        ) => {
                                            let MintNonFungibleResourceEvent { ids } =
                                                scrypto_decode(event_data).expect("Must succeed!");
                                            acc.extend(ids.into_iter().map(|value| {
                                                NonFungibleGlobalId::new(resource_address, value)
                                            }));
                                            acc
                                        }
                                        (
                                            Ok(resource_address),
                                            BurnNonFungibleResourceEvent::EVENT_NAME,
                                        ) => {
                                            let BurnNonFungibleResourceEvent { ids } =
                                                scrypto_decode(event_data).expect("Must succeed!");
                                            ids.into_iter()
                                                .map(|value| {
                                                    NonFungibleGlobalId::new(
                                                        resource_address,
                                                        value,
                                                    )
                                                })
                                                .for_each(|global_id| {
                                                    acc.shift_remove(&global_id);
                                                });
                                            acc
                                        }
                                        _ => acc,
                                    }
                                }
                                Emitter::Method(..) | Emitter::Function(..) => acc,
                            }
                        },
                    ),
                },
                worktop_changes: execution_trace.worktop_changes(),
                fee_summary: FeeSummary {
                    execution_fees_in_xrd: fee_summary.total_execution_cost_in_xrd,
                    finalization_fees_in_xrd: fee_summary.total_finalization_cost_in_xrd,
                    storage_fees_in_xrd: fee_summary.total_storage_cost_in_xrd,
                    royalty_fees_in_xrd: fee_summary.total_royalty_cost_in_xrd,
                },
                locked_fees: LockedFees {
                    contingent: execution_trace.fee_locks.contingent_lock,
                    non_contingent: execution_trace.fee_locks.lock,
                },
            }),
            TransactionReceipt {
                result:
                    TransactionResult::Commit(CommitResult {
                        outcome: TransactionOutcome::Success(..),
                        execution_trace: None,
                        ..
                    }),
                ..
            } => Err(ToolkitReceiptError::ReceiptLacksExecutionTrace),
            TransactionReceipt {
                result:
                    TransactionResult::Commit(CommitResult {
                        outcome: TransactionOutcome::Failure(error),
                        ..
                    }),
                ..
            } => Ok(Self::CommitFailure {
                reason: format!("{error:?}"),
            }),
            TransactionReceipt {
                result: TransactionResult::Reject(error),
                ..
            } => Ok(Self::Reject {
                reason: format!("{error:?}"),
            }),
            TransactionReceipt {
                result: TransactionResult::Abort(error),
                ..
            } => Ok(Self::Abort {
                reason: format!("{error:?}"),
            }),
        }
    }
}

fn get_metadata_updates(
    state_updates: &StateUpdates,
) -> IndexMap<NodeId, IndexMap<String, MetadataUpdate<RuntimeTypeSelector>>> {
    let desired_partition =
        MetadataPartitionOffset::EntryKeyValue.as_partition(METADATA_BASE_PARTITION);
    state_updates
        .by_node
        .iter()
        .flat_map(|(node_id, NodeStateUpdates::Delta { by_partition })| {
            by_partition
                .get(&desired_partition)
                .map(|partition_state_updates| (node_id, partition_state_updates))
        })
        .fold(
            index_map_new(),
            |mut acc, (node_id, partition_state_updates)| {
                acc.entry(*node_id).or_default().extend(
                    partition_state_updates
                        .iter_map_entries()
                        .map(|(key, database_update)| {
                            let key =
                                scrypto_decode::<MetadataEntryKeyPayload>(key).expect("Can't fail");
                            let value = match database_update {
                                DatabaseUpdateRef::Set(value) => {
                                    let value = scrypto_decode::<MetadataEntryEntrySubstate>(value)
                                        .expect("Can't fail")
                                        .into_value();
                                    match value {
                                        Some(value) => MetadataUpdate::Set(
                                            value.fully_update_and_into_latest_version(),
                                        ),
                                        None => MetadataUpdate::Delete,
                                    }
                                }
                                DatabaseUpdateRef::Delete => MetadataUpdate::Delete,
                            };
                            (key.content, value)
                        }),
                );
                acc
            },
        )
}
