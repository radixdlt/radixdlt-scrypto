use std::collections::HashMap;
use std::path::PathBuf;

use radix_engine::engine::ScryptoInterpreter;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::*;
use radix_engine::{ledger::*, wasm::WasmEngine};
use radix_engine_interface::{api::types::RENodeId, data::ScryptoDecode};
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};

pub struct RadixEngineDB {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RadixEngineDB {
    pub fn new(root: PathBuf) -> Self {
        let db = DB::open_default(root.as_path()).unwrap();
        Self { db }
    }

    pub fn with_bootstrap<W: WasmEngine>(
        root: PathBuf,
        scrypto_interpreter: &ScryptoInterpreter<W>,
    ) -> Self {
        let mut substate_store = Self::new(root);
        bootstrap(&mut substate_store, scrypto_interpreter);
        substate_store
    }

    pub fn list_packages(&self) -> Vec<PackageAddress> {
        let start = &scrypto_encode(&SubstateId(
            RENodeId::Global(GlobalAddress::Package(PackageAddress::Normal([0; 26]))),
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            RENodeId::Global(GlobalAddress::Package(PackageAddress::Normal([255; 26]))),
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);
        substate_ids
            .into_iter()
            .map(|id| {
                if let SubstateId(
                    RENodeId::Global(GlobalAddress::Package(package_address)),
                    SubstateOffset::Global(GlobalOffset::Global),
                ) = id
                {
                    package_address
                } else {
                    panic!("Expected a package global substate id.")
                }
            })
            .collect()
    }

    fn list_components_helper(
        &self,
        start: ComponentAddress,
        end: ComponentAddress,
    ) -> Vec<ComponentAddress> {
        let start = &scrypto_encode(&SubstateId(
            RENodeId::Global(GlobalAddress::Component(start)),
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            RENodeId::Global(GlobalAddress::Component(end)),
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);
        substate_ids
            .into_iter()
            .map(|id| {
                if let SubstateId(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Global(GlobalOffset::Global),
                ) = id
                {
                    component_address
                } else {
                    panic!("Expected a component global substate id.")
                }
            })
            .collect()
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
            RENodeId::Global(GlobalAddress::Resource(ResourceAddress::Normal([0; 26]))),
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            RENodeId::Global(GlobalAddress::Resource(ResourceAddress::Normal([255; 26]))),
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);
        substate_ids
            .into_iter()
            .map(|id| {
                if let SubstateId(
                    RENodeId::Global(GlobalAddress::Resource(resource_address)),
                    SubstateOffset::Global(GlobalOffset::Global),
                ) = id
                {
                    resource_address
                } else {
                    panic!("Expected a resource manager global substate id.")
                }
            })
            .collect()
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

impl QueryableSubstateStore for RadixEngineDB {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        let unit = scrypto_encode(&()).unwrap();
        let id = scrypto_encode(&SubstateId(
            RENodeId::KeyValueStore(kv_store_id.clone()),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                scrypto_encode(&unit).unwrap(),
            )),
        ))
        .unwrap();

        let mut iter = self
            .db
            .iterator(IteratorMode::From(&id, Direction::Forward));
        let mut items = HashMap::new();
        while let Some(kv) = iter.next() {
            let (key, value) = kv.unwrap();
            let substate: OutputValue = scrypto_decode(&value.to_vec()).unwrap();
            let substate_id: SubstateId = scrypto_decode(&key).unwrap();
            if let SubstateId(
                RENodeId::KeyValueStore(id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) = substate_id
            {
                if id == *kv_store_id {
                    items.insert(key, substate.substate)
                } else {
                    break;
                }
            } else {
                break;
            };
        }
        items
    }
}

impl ReadableSubstateStore for RadixEngineDB {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.read(substate_id)
            .map(|b| scrypto_decode(&b).expect("Could not decode persisted substate"))
    }
}

impl WriteableSubstateStore for RadixEngineDB {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.write(
            substate_id,
            scrypto_encode(&substate).expect("Could not encode substate for persistence"),
        );
    }
}
