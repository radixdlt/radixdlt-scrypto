# Radix Engine

Radix Engine is the underlying execution engine specialized to run DeFi-based Scrypto applications.

## Architecture

Radix Engine is organized into 7 layers. Each layer provides a Downstream API to the layer above. Execution
layers provide an Upstream API which the layer above must implement.

| Layer          | Responsibilities                                                                   | Downstream API                                                                                                                                                                                                                                                               |     | Implementation(s)                                                                                          |
|----------------|------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----|------------------------------------------------------------------------------------------------------------|
| Blueprints     | Application Logic (e.g. Blueprints written in Scrypto)                             |                                                                                                                                                                                                                                                                              |     | [Native Blueprints](src/blueprints)<br>[Scrypto Blueprints](../radix-engine-tests/tests/blueprints)        | 
| VM             | Application Execution                                                              | WASM + [Scrypto API](../scrypto/src/engine/scrypto_env.rs)                                                                                                                                                                                                                   |     | [ScryptoVM](src/vm/scrypto_vm.rs)<br>[NativeVM](src/vm/native_vm.rs)                                       |
| System         | Type Checking<br>Package/Blueprint/Object semantics<br>Application Standardization | [Substate API](../radix-engine-interface/src/api/substate_api.rs)<br>[Object API](../radix-engine-interface/src/api/object_api.rs)<br>[Blueprint API](../radix-engine-interface/src/api/blueprint_api.rs)<br>[Costing API](../radix-engine-interface/src/api/costing_api.rs) |     | [System](src/system)                                                                                       |
| Kernel         | Call Frame Logic<br>Ownership/Reference handling                                   | [Kernel API](src/kernel/kernel_api.rs)                                                                                                                                                                                                                                       |     | [Kernel](src/kernel)                                                                                       |
| Virtualization | Node virtualization<br>Substate virtualization                                     | Virtualized Store                                                                                                                                                                                                                                                            |     | [Node Virtualization](src/system/system_modules/virtualization)                                            |
| Track          | Dependency and Update Tracking<br>Proof and Receipt Creation                       | [Substate Store](../radix-engine-stores/src/interface.rs)                                                                                                                                                                                                                    |     | [Track](src/track)                                                                                         |
| Database       | Runtime Read-Only Physical Storage                                                 | [Substate Database](../radix-engine-stores/src/interface.rs)                                                                                                                                                                                                                 |     | [InMemoryDB](../radix-engine-stores/src/memory_db.rs)<br>[RocksDB](../radix-engine-stores/src/rocks_db.rs) |

## State Model 

A Substate Entry is the smallest atomic unit of state in the Radix Engine. It is a key value
entry which represents an entry in a KV Database or a leaf node in a State Merkle Tree structure.

|       | NodeId | ModuleId | SubstateKey | Substate Value |
|-------|--------|----------|-------------|----------------|
| Bytes | 27     | 1        | Variable    | Variable       |