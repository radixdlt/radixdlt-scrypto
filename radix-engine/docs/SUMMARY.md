# Radix Engine

[Overview](README.md)

# Architecture

- [Layered Architecture](architecture/layers.md)
  - [Application Layer](architecture/layers/application/README)
    - [Blueprint Definition](architecture/layers/application/blueprint_definition.md)
      - [Blueprint Schema](architecture/layers/application/blueprint_schema.md)
    - [WASM Environment](architecture/layers/application/wasm_environment.md)
  - [VM Layer](architecture/layers/vm.md)
  - [System Layer](architecture/layers/system.md)
    - Type System (TODO)
    - System Modules (TODO)
    - Object Modules (TODO)
  - [Kernel Layer](architecture/layers/kernel.md)
    - Move/Borrow Checking (TODO)
  - [Database Layer](architecture/layers/database.md)
- Data Architecture (TODO)
  - Substates / SBOR (TODO)

# Execution

- [Transaction Lifecycle](execution/lifecycle.md)
  - [Bootup](execution/bootup.md)
  - Runtime (TODO)
    - System Calls (TODO)
    - Invocations (TODO)
  - [Shutdown](execution/shutdown.md)
- Substate Flashing (TODO)
- Genesis Bootstrap (TODO)
- Protocol Updates (TODO)
 
# Native Systems

- [Access Control](native/access_control/README)
  - [Role Definition](native/access_control/role_definition.md)
  - [Role Assignment](native/access_control/role_assignment.md)
  - [Auth Zone](native/access_control/authzone.md)
  - [Authorization](native/access_control/authorization.md)
- Transaction Manifest (TODO)
- Resources (TODO)
- Royalties (TODO)
- Metadata (TODO)
