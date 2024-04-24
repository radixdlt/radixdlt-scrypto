# Radix Engine

Radix Engine is a transaction-based state machine that updates the ledger state by incrementally
executing transactions.

Unlike Ethereum where the ledger state is a flat mapping between addresses and account states, Radix
Engine organizes its state into a forest of objects. Child objects are exclusively owned by its
parent in the tree hierarchy. Each root object is assigned a global address.

Every object has an associated blueprint, which defines logic for updating the object's internal
state. Multiple blueprints can be packed into a package and published as a single unit.

A set of native packages are defined by Radix Engine which form built-in system standards such as
accounts, access control, resources, etc.