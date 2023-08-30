# Radix Engine

Radix Engine is the underlying execution engine designed to run DeFi-based Scrypto applications.

The architecture is heavily influenced by traditional Kernel design (though much simplified) and Rust's
Ownership and Type-Checking paradigms (also much simplified). The novel idea here is in combining
ideas from the two worlds, Operating System and Language, or simply "Implement Rust
Semantics at the System Layer".

## Architecture

Radix Engine execution is organized into 5 layers, each layer providing an API to the layer above.

Execution layers may also optionally provide a Callback API which the layer above must implement.

| Execution Layer | Layer ID | Description             | Responsibilities                                                                                                                                    | API                                                                                                                                                                                                                                                                              | Callback API                                             | Implementation                                                                                             |
|-----------------|----------|-------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|------------------------------------------------------------------------------------------------------------|
| Application     | 5        | "User Space"            | Application Logic (e.g. Blueprints written in Scrypto)                                                                                              |                                                                                                                                                                                                                                                                                  |                                                          | [Native Blueprints](src/blueprints)<br>[Scrypto Blueprints](../radix-engine-tests/tests/blueprints)        | 
| VM              | 4        | "Virtual CPU"           | Application Execution                                                                                                                               | WASM + [Scrypto API](../scrypto/src/engine/scrypto_env.rs)                                                                                                                                                                                                                       |                                                          | [VM](src/vm)                                                                                               |
| System          | 3        | "Operating System"      | Type Checking<br>Package/Blueprint/Object semantics<br>Application Standardization (e.g. Authorization, Versioning)                                 | [Substate API](../radix-engine-interface/src/api/locked_substate_api)<br>[Object API](../radix-engine-interface/src/api/object_api.rs)<br>[Blueprint API](../radix-engine-interface/src/api/blueprint_api.rs)<br>[Costing API](../radix-engine-interface/src/api/costing_api.rs) | [System Callback API](src/system/system_callback_api.rs) | [System](src/system)                                                                                       |
| Kernel          | 2        | "I/O Device Management" | Call Frame Message Passing<br>Ownership/Reference handling<br>State Virtualization Mechanism<br>Substate Device Management<br>Transaction Execution | [Kernel API](src/kernel/kernel_api.rs)                                                                                                                                                                                                                                           | [Kernel Callback API](src/kernel/kernel_callback_api.rs) | [Kernel](src/kernel)                                                                                       |
| Database        | 1        | "Persistence"           | Runtime Read-Only Physical Storage                                                                                                                  | [Substate Database](../radix-engine-store-interface/src/interface.rs)                                                                                                                                                                                                            |                                                          | [InMemoryDB](../radix-engine-stores/src/memory_db.rs)<br>[RocksDB](../radix-engine-stores/src/rocks_db.rs) |


## Data Abstraction

If looked at from a purely state perspective, the layers can be reduced to the following 4 layers:

| Data Layer  | Layer ID | Abstraction                                                                                |
|-------------|----------|--------------------------------------------------------------------------------------------|
| Application | 5        | Application Interface (e.g. Amount of money in my account)                                 |
| System      | 3        | Package/Blueprint/Object semantics<br>Blueprint Fields and Collections<br>Blueprint Typing |
| Kernel      | 2        | Node/Partition/Substate semantics<br>Substate Ownership/References<br>                     |
| Database    | 1        | Key/Value Database                                                                         |
