# Architecture

The Radix Engine architecture is influenced by traditional Kernel design and Rust's Ownership
and Type-Checking paradigms. The novel idea here is in combining ideas from the two worlds,
Operating System and Language, or simply "Implement Rust Semantics at the System Layer".

Radix Engine is organized into 5 layers. Each layer has specific responsibilities and
provides an API to the layer above. Middle layers also provide a Callback API which the
layer above must implement.

| Layer Name  | Layer ID | Responsibilities                                                                                                                                    | API                                                                                                                                                                                                     | Callback API                                             | Implementation                                                                                                           |
|-------------|----------|-----------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| Application | 5        | Application Logic (e.g. Blueprints written in Scrypto)                                                                                              |                                                                                                                                                                                                         |                                                          | [Native Blueprints](src/blueprints)<br>[Scrypto Blueprints](../radix-engine-tests/tests/blueprints)                      | 
| VM          | 4        | Application Execution                                                                                                                               | [Scrypto VM API](../scrypto/src/engine/scrypto_env.rs)                                                                                                                                                  |                                                          | [VM](src/vm)                                                                                                             |
| System      | 3        | Type Checking<br>Package/Blueprint/Object semantics<br>Application Standardization (e.g. Authorization, Versioning)                                 | [Object API](../radix-engine-interface/src/api/object_api.rs)<br>[Blueprint API](../radix-engine-interface/src/api/blueprint_api.rs)<br>[Costing API](../radix-engine-interface/src/api/costing_api.rs) | [System Callback API](src/system/system_callback_api.rs) | [System](src/system)                                                                                                     |
| Kernel      | 2        | Call Frame Message Passing<br>Ownership/Reference handling<br>State Virtualization Mechanism<br>Substate Device Management<br>Transaction Execution | [Kernel API](src/kernel/kernel_api.rs)                                                                                                                                                                  | [Kernel Callback API](src/kernel/kernel_callback_api.rs) | [Kernel](src/kernel)                                                                                                     |
| Database    | 1        | Runtime Read-Only Physical Storage                                                                                                                  | [Substate Database](../radix-substate-store-interface/src/interface.rs)                                                                                                                                 |                                                          | [InMemoryDB](../radix-substate-store-impls/src/memory_db.rs)<br>[RocksDB](../radix-substate-store-impls/src/rocks_db.rs) |

## Application

The application layer is responsible for 

## VM

## System

## Kernel

## Database
