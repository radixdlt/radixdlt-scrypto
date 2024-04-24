# Radix Engine

[Overview](README.md)

# Architecture

- [Layered Architecture](architecture/layers.md)
  - [Application Layer](architecture/layers/application/README)
    - [Blueprint Definition](architecture/layers/application/blueprint_definition.md)
      - Inner vs. Outer Blueprint
      - Transience
      - Features
      - Generics
      - [State](architecture/layers/application/state.md)
      - Events (TODO)
      - [Functions](architecture/layers/application/functions.md)
      - Types (TODO)
    - [WASM Environment](architecture/layers/application/wasm_environment.md)
    - Type System (TODO)
  - [VM Layer](architecture/layers/vm.md)
  - [System Layer](architecture/layers/system.md)
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

- [Authorization](native/access_control/README)
  - [Role Definition](native/access_control/role_definition.md)
  - [Role Assignment](native/access_control/role_assignment.md)
  - [Auth Zone](native/access_control/authzone.md)
  - [Authorization Flow](native/access_control/authorization.md)
- Transaction Manifest (TODO)
- Resources (TODO)
- Royalties (TODO)
- Metadata (TODO)
