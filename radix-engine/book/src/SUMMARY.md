# Radix Engine

[Introduction](README.md)

# Architecture

- [Layered Architecture](architecture/layers.md)
- [Application Layer](architecture/layers/application/README.md)
  - [Blueprint Definition](architecture/layers/application/blueprint_definition.md)
    - [Inner vs. Outer Blueprint]()
    - [Transience]()
    - [Features]()
    - [Generics]()
    - [State](architecture/layers/application/state.md)
    - [Events]()
    - [Functions](architecture/layers/application/functions.md)
    - [Types]()
  - [WASM Environment](architecture/layers/application/wasm_environment.md)
  - [Type System]()
- [VM Layer](architecture/layers/vm.md)
- [System Layer](architecture/layers/system.md)
  - [System Modules]()
  - [Object Modules]()
- [Kernel Layer](architecture/layers/kernel.md)
  - [Move/Borrow Checking]()
- [Database Layer](architecture/layers/database.md)
- [Data Architecture]()
  - [Substates / SBOR]()

# Execution

- [Transaction Lifecycle](execution/lifecycle.md)
  - [Bootup](execution/bootup.md)
  - [Runtime]()
    - [System Calls]()
    - [Invocations]()
  - [Shutdown](execution/shutdown.md)
- [Substate Flashing]()
- [Genesis Bootstrap]()
- [Protocol Updates]()
 
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
