# Radix Engine

[Introduction](README.md)

# Architecture

- [Layered Architecture](architecture/layers.md)
- [Application Layer](architecture/application/README.md)
  - [Blueprint](architecture/application/blueprint/README.md)
    - [Inner and Outer Blueprints](architecture/application/blueprint/inner_outer.md)
    - [Transience](architecture/application/blueprint/transience.md)
    - [Features](architecture/application/blueprint/features.md)
    - [Generics](architecture/application/blueprint/generics.md)
    - [Fields](architecture/application/blueprint/fields.md)
    - [Collections](architecture/application/blueprint/collections.md)
    - [Events](architecture/application/blueprint/events.md)
    - [Functions]()
    - [Types](architecture/application/blueprint/types.md)
    - [Blueprint Modules]()
  - [Object]()
    - [Object Modules]()
  - [Type System]()
- [VM Layer](architecture/vm.md)
- [System Layer](architecture/system.md)
  - [System Modules]()
- [Kernel Layer](architecture/kernel.md)
  - [Move/Borrow Checking]()
- [Database Layer](architecture/database.md)
- [Data Architecture]()
  - [Substates / SBOR]()

# Execution

- [Transaction Lifecycle](execution/lifecycle.md)
  - [Bootup](execution/bootup.md)
  - [Runtime]()
  - [Shutdown](execution/shutdown.md)
- [WASM Environment](architecture/application/wasm_environment.md)
- [Object Lifecycle]()
  - [Instantiation]()
  - [State Reads/Writes]()
  - [Destruction]()
- [Invocations]()

# Native Systems

- [Authorization](native/access_control/README)
  - [Role Definition](native/access_control/role_definition.md)
  - [Role Assignment](native/access_control/role_assignment.md)
  - [Auth Zone](native/access_control/authzone.md)
  - [Authorization Flow](native/access_control/authorization.md)
- [Transaction Manifest]()
- [Resources]()
- [Royalties]()
- [Metadata]()

# Protocol
- [Genesis Bootstrap]()
- [Protocol Updates]()
 