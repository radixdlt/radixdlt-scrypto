use std::collections::HashMap;
use std::path::PathBuf;

use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::*;
use radix_engine::{ledger::*, wasm::WasmEngine};
use radix_engine_common::types::NodeId;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};

pub struct RocksdbSubstateStore {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksdbSubstateStore {
    pub fn new(root: PathBuf) -> Self {
        let db = DB::open_default(root.as_path()).unwrap();
        Self { db }
    }

    pub fn with_bootstrap<W: WasmEngine>(
        root: PathBuf,
        scrypto_interpreter: &ScryptoInterpreter<W>,
    ) -> Self {
        let mut substate_db = Self::new(root);
        bootstrap(&mut substate_db, scrypto_interpreter);
        substate_db
    }

    pub fn commit(&mut self, state_diff: &StateDiff) -> CommitReceipt {
        let mut receipt = CommitReceipt::new();

        for output_id in &self.down_substates {
            receipt.down(output_id.clone());
        }
        for (substate_id, output_value) in &self.up_substates {
            let output_id = OutputId {
                substate_id: substate_id.clone(),
                substate_hash: hash(
                    scrypto_encode(&output_value.substate).unwrap_or_else(|err| {
                        panic!(
                            "Could not encode newly-committed substate: {:?}. Substate: {:?}",
                            err, &output_value.substate
                        )
                    }),
                ),
                version: output_value.version,
            };
            receipt.up(output_id);
            store.put_substate(substate_id.clone(), output_value.clone());
        }

        receipt
    }

    pub fn list_packages(&self) -> Vec<PackageAddress> {
        let start = &scrypto_encode(&SubstateId(
            NodeId::GlobalObject(PackageAddress::Normal([0; 26]).into()),
            TypedModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            NodeId::GlobalObject(PackageAddress::Normal([255; 26]).into()),
            TypedModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);

        let mut addresses = Vec::new();
        for substate_id in substate_ids {
            if let SubstateId(
                NodeId::GlobalObject(Address::Package(package_address)),
                TypedModuleId::TypeInfo,
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
            ) = substate_id
            {
                addresses.push(package_address);
            }
        }

        addresses
    }

    fn list_components_helper(
        &self,
        start: ComponentAddress,
        end: ComponentAddress,
    ) -> Vec<ComponentAddress> {
        let start = &scrypto_encode(&SubstateId(
            NodeId::GlobalObject(Address::Component(start)),
            TypedModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            NodeId::GlobalObject(Address::Component(end)),
            TypedModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);
        let mut addresses = Vec::new();
        for substate_id in substate_ids {
            if let SubstateId(
                NodeId::GlobalObject(Address::Component(component_address)),
                TypedModuleId::TypeInfo,
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
            ) = substate_id
            {
                addresses.push(component_address);
            }
        }

        addresses
    }

    pub fn list_components(&self) -> Vec<ComponentAddress> {
        let mut addresses = Vec::new();
        addresses.extend(self.list_components_helper(
            ComponentAddress::Account([0u8; 26]),
            ComponentAddress::Account([255u8; 26]),
        ));
        addresses.extend(self.list_components_helper(
            ComponentAddress::Normal([0u8; 26]),
            ComponentAddress::Normal([255u8; 26]),
        ));
        addresses
    }

    pub fn list_resource_managers(&self) -> Vec<ResourceAddress> {
        let start = &scrypto_encode(&SubstateId(
            NodeId::GlobalObject(ResourceAddress::Fungible([0; 26]).into()),
            TypedModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            NodeId::GlobalObject(ResourceAddress::NonFungible([255; 26]).into()),
            TypedModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);
        let mut addresses = Vec::new();
        for substate_id in substate_ids {
            if let SubstateId(
                NodeId::GlobalObject(Address::Resource(resource_address)),
                TypedModuleId::TypeInfo,
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
            ) = substate_id
            {
                addresses.push(resource_address);
            }
        }

        addresses
    }

    fn list_items<T: ScryptoDecode>(&self, start: &[u8], inclusive_end: &[u8]) -> Vec<T> {
        let mut iter = self
            .db
            .iterator(IteratorMode::From(start, Direction::Forward));
        let mut items = Vec::new();
        while let Some(kv) = iter.next() {
            let (key, _value) = kv.unwrap();
            if key.as_ref() > inclusive_end {
                break;
            }
            if key.len() == start.len() {
                items.push(scrypto_decode(key.as_ref()).unwrap());
            }
        }
        items
    }

    fn read(&self, substate_id: &SubstateId) -> Option<Vec<u8>> {
        // TODO: Use get_pinned
        self.db
            .get(scrypto_encode(substate_id).expect("Could not encode substate id"))
            .unwrap()
    }

    fn write(&self, substate_id: SubstateId, value: Vec<u8>) {
        self.db
            .put(
                scrypto_encode(&substate_id).expect("Could not encode substate id"),
                value,
            )
            .unwrap();
    }
}

impl QueryableSubstateStore for RocksdbSubstateStore {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        let mut iter = self.db.iterator(IteratorMode::Start);
        let mut items = HashMap::new();
        while let Some(kv) = iter.next() {
            let (key, value) = kv.unwrap();
            let substate_id: SubstateId = scrypto_decode(&key).unwrap();
            if let SubstateId(
                NodeId::KeyValueStore(id),
                TypedModuleId::ObjectState,
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(entry_id)),
            ) = substate_id
            {
                let substate: OutputValue = scrypto_decode(&value.to_vec()).unwrap();
                if id == *kv_store_id {
                    items.insert(entry_id, substate.substate);
                }
            }
        }
        items
    }
}

impl ReadableSubstateStore for RocksdbSubstateStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.read(substate_id)
            .map(|b| scrypto_decode(&b).expect("Could not decode persisted substate"))
    }
}

impl CommittableSubstateDatabase for RocksdbSubstateStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.write(
            substate_id,
            scrypto_encode(&substate).expect("Could not encode substate for persistence"),
        );
    }
}
