# Radix Engine

[Overview](README.md)

# Architecture

- [Layered Architecture](architecture/layers.md)
  - [Application Layer](architecture/layers/application.md)
    - [WASM environment](architecture/layers/application/wasm_environment.md)
    - Native environment (TODO)
    - Blueprint Definition (TODO)
  - [VM Layer](architecture/layers/vm.md)
  - [System Layer](architecture/layers/system.md)
    - System Modules (TODO)
    - Object Modules (TODO)
  - [Kernel Layer](architecture/layers/kernel.md)
  - [Database Layer](architecture/layers/database.md)
- Data Architecture (TODO)
  - Substates / SBOR (TODO)

# Execution

- [Transaction Lifecycle](execution/lifecycle.md)
  - [Bootup](execution/bootup.md)
  - Runtime (TODO)
    - System Calls (TODO)
    - Invocations (TODO)
    - Move/Borrow Checking (TODO)
  - [Shutdown](execution/shutdown.md)
- Bootstrapping (TODO)
- Protocol Updates (TODO)
 
# Native Systems

- Transaction Manifest
- Type System
- Resources
- [Access Control](native/access_control/README)
  - [Role Definition](native/access_control/role_definition.md)
  - [Role Assignment](native/access_control/role_assignment.md)
  - [Auth Zone](native/access_control/authzone.md)
  - [Authorization](native/access_control/authorization.md)
- Royalties
- Metadata
