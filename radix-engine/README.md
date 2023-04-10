# Radix Engine

Radix Engine is the underlying execution engine specialized to run DeFi-based Scrypto applications.

## Architecture

Radix Engine is organized into 7 layers, from high level to low level. Each layer provides an API to
the layer above.

| Layer                        | Responsibilities                                    | Provides API                                                      |
|------------------------------|-----------------------------------------------------|-------------------------------------------------------------------|
| [Blueprints](src/blueprints) | Application Logic                                   |                                                                   | 
| VM                           | Application Execution                               | WASM + Scrypto API                                                |
| System                       | Type Checking<br>Package/Blueprint/Object semantics | Actor API<br>Object API<br>Substate API<br>                       |
| Kernel                       | Call Frame Logic<br>Ownership/Reference handling    | Kernel Node API<br> Kernel Substate API<br> Kernel Invoke API<br> |
| Virtualization               | Node virtualization<br>Substate virtualization      | Virtualized Store                                                 |
| Track                        | Dependency and Update Tracking<br>Receipt Creation  | Substate Store                                                    |
| Database                     | Runtime Read-Only Physical Storage                  | Substate Database                                                 |
