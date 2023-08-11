# Radix Engine

Radix Engine is the underlying execution engine designed to run DeFi-based Scrypto applications.

## Execution

Radix Engine execution is organized into 6 layers, each layer providing an API to the layer above.

Execution layers may also optionally provide a Callback API which the layer above must implement.

| Execution Layer | Layer ID | Responsibilities                                                                                                                | API                                                                                                                                                                                                                                                                              | Callback API                                             | Implementation                                                                                             |
|-----------------|----------|---------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|------------------------------------------------------------------------------------------------------------|
| Application     | 3        | Application Logic (e.g. Blueprints written in Scrypto)                                                                          |                                                                                                                                                                                                                                                                                  |                                                          | [Native Blueprints](src/blueprints)<br>[Scrypto Blueprints](../radix-engine-tests/tests/blueprints)        | 
| VM              | 2.5      | Application Execution                                                                                                           | WASM + [Scrypto API](../scrypto/src/engine/scrypto_env.rs)                                                                                                                                                                                                                       |                                                          | [VM](src/vm)                                                                                               |
| System          | 2        | Type Checking<br>Package/Blueprint/Object semantics<br>Application Standardization                                              | [Substate API](../radix-engine-interface/src/api/locked_substate_api)<br>[Object API](../radix-engine-interface/src/api/object_api.rs)<br>[Blueprint API](../radix-engine-interface/src/api/blueprint_api.rs)<br>[Costing API](../radix-engine-interface/src/api/costing_api.rs) | [System Callback API](src/system/system_callback_api.rs) | [System](src/system)                                                                                       |
| Kernel          | 1        | Call Frame Message Passing<br>Ownership/Reference handling<br>State Virtualization Mechanism<br>Heap/Substate Device Management | [Kernel API](src/kernel/kernel_api.rs)                                                                                                                                                                                                                                           | [Kernel Callback API](src/kernel/kernel_callback_api.rs) | [Kernel](src/kernel)                                                                                       |
| Track           | 0.5      | Dependency and Update Tracking<br>Proof and Receipt Creation                                                                    | [Substate Store](src/track/interface.rs)                                                                                                                                                                                                                                         |                                                          | [Track](src/track)                                                                                         |
| Database        | 0        | Runtime Read-Only Physical Storage                                                                                              | [Substate Database](../radix-engine-store-interface/src/interface.rs)                                                                                                                                                                                                            |                                                          | [InMemoryDB](../radix-engine-stores/src/memory_db.rs)<br>[RocksDB](../radix-engine-stores/src/rocks_db.rs) |


## Data

Each of the execution layers serves to manipulate state during runtime. State is abstracted with the following layers.

| Data Layer  | Layer ID | Responsibilities                                                       |
|-------------|----------|------------------------------------------------------------------------|
| Application | 3        | Application Data (e.g. Amount of money in my account)                  |
| System      | 2        | Package/Blueprint/Object semantics<br>Blueprint Typing                 |
| Kernel      | 1        | Node/Partition/Substate semantics<br>Substate Ownership/References<br> |
| Database    | 0        | Key/Value Database                                                     |
