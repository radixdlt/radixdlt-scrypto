# Kernel Layer

The kernel layer is responsible for:
* Defining the Node/Partition/Substate abstraction
* Defining the Call Frame abstraction
* Maintaining Ownership/Reference invariants
* Managing transaction state updates, which are to be subsequently committed to the
  database at the end of the transaction

## Implementation

The kernel layer is implemented on top of the database layer's Partition Key and Sort Key
abstractions.