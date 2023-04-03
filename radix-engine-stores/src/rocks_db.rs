use std::collections::HashMap;
use std::path::PathBuf;

use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::system::node_substates::{PersistedSubstate, RuntimeSubstate};
use radix_engine::types::*;
use radix_engine::{ledger::*, wasm::WasmEngine};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::data::scrypto::ScryptoDecode;
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
            RENodeId::GlobalObject(PackageAddress::Normal([0; 26]).into()),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            RENodeId::GlobalObject(PackageAddress::Normal([255; 26]).into()),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);

        let mut addresses = Vec::new();
        for substate_id in substate_ids {
            if let SubstateId(
                RENodeId::GlobalObject(Address::Package(package_address)),
                NodeModuleId::TypeInfo,
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
            RENodeId::GlobalObject(Address::Component(start)),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            RENodeId::GlobalObject(Address::Component(end)),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);
        let mut addresses = Vec::new();
        for substate_id in substate_ids {
            if let SubstateId(
                RENodeId::GlobalObject(Address::Component(component_address)),
                NodeModuleId::TypeInfo,
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
            RENodeId::GlobalObject(ResourceAddress::Fungible([0; 26]).into()),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let end = &scrypto_encode(&SubstateId(
            RENodeId::GlobalObject(ResourceAddress::NonFungible([255; 26]).into()),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .unwrap();
        let substate_ids: Vec<SubstateId> = self.list_items(start, end);
        let mut addresses = Vec::new();
        for substate_id in substate_ids {
            if let SubstateId(
                RENodeId::GlobalObject(Address::Resource(resource_address)),
                NodeModuleId::TypeInfo,
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

impl QueryableSubstateStore for RadixEngineDB {
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
                RENodeId::KeyValueStore(id),
                NodeModuleId::SELF,
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

impl ReadableSubstateStore for RadixEngineDB {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.read(substate_id)
            .map(|b| scrypto_decode(&b).expect("Could not decode persisted substate"))
    }

    fn first_in_iterable(
        &self,
        node_id: &RENodeId,
        module_id: NodeModuleId,
        mut count: u32,
    ) -> Vec<(SubstateId, RuntimeSubstate)> {
        // FIXME: Super hack!
        let start = SubstateId(
            node_id.clone(),
            module_id,
            SubstateOffset::Component(ComponentOffset::State0),
        );
        let start = scrypto_encode(&start).unwrap();

        let mut iter = self
            .db
            .iterator(IteratorMode::From(&start, Direction::Forward));

        let mut items = Vec::new();
        while let Some(kv) = iter.next() {
            if count == 0u32 {
                break;
            }

            let (key, value) = kv.unwrap();
            let id: SubstateId = scrypto_decode(key.as_ref()).unwrap();
            if !id.0.eq(node_id) || !id.1.eq(&module_id) {
                break;
            }

            let output_value: OutputValue = scrypto_decode(value.as_ref()).unwrap();

            items.push((id, output_value.substate.to_runtime()));
            count -= 1;
        }

        items
    }
}

impl WriteableSubstateStore for RadixEngineDB {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.write(
            substate_id,
            scrypto_encode(&substate).expect("Could not encode substate for persistence"),
        );
    }

    fn remove_substate(&mut self, substate_id: &SubstateId) {
        let key = scrypto_encode(substate_id).unwrap();
        self.db.delete(key).unwrap();
    }
}
