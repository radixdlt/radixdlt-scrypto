# Radix Engine

Radix Engine is the underlying execution engine specialized to run DeFi-based Scrypto applications.

## Architecture

Radix Engine is organized into 7 layers, from high level to low level. Each layer provides an API to
the layer above.

| Layer          | Responsibilities                                             | Provides API                                                                                                                 | Implementation(s)                                                                                          |
|----------------|--------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------|
| Blueprints     | Application Logic                                            |                                                                                                                              | [Native Blueprints](src/blueprints)<br>[Scrypto Blueprints](../radix-engine-tests/tests/blueprints)        | 
| VM             | Application Execution                                        | WASM + Scrypto API                                                                                                           | [ScryptoVM](src/vm/scrypto_vm.rs)<br>[NativeVM](src/vm/native_vm.rs)                                       |
| System         | Type Checking<br>Package/Blueprint/Object semantics          | [Substate API](src/radix-engine-interface/api/substate_api.rs)<br>[Object API](src/radix-engine-interface/api/object_api.rs) | [System](src/system)                                                                                       |
| Kernel         | Call Frame Logic<br>Ownership/Reference handling             | [Kernel API](src/kernel/kernel_api.rs)                                                                                       | [Kernel](src/kernel)                                                                                       |
| Virtualization | Node virtualization<br>Substate virtualization               | Virtualized Store                                                                                                            | [Node Virtualization](src/system/system_modules/virtualization)                                            |
| Track          | Dependency and Update Tracking<br>Proof and Receipt Creation | [Substate Store](../radix-engine-stores/src/interface.rs)                                                                    | [Track](src/track)                                                                                         |
| Database       | Runtime Read-Only Physical Storage                           | [Substate Database](../radix-engine-stores/src/interface.rs)                                                                 | [InMemoryDB](../radix-engine-stores/src/memory_db.rs)<br>[RocksDB](../radix-engine-stores/src/rocks_db.rs) |
