# Object Model

Unlike Ethereum where the ledger state is a flat mapping between addresses and account states, Radix
Engine organizes its state into a forest of *objects*. Child objects are exclusively owned by its
parent in the tree hierarchy. Each root object is assigned a *global address*.

![](object_model.drawio.svg)

Each object has:
* A [BlueprintId](blueprint_id.md) (type)
* An optional [Outer Object](inner_outer_objects.md)
* A list of [Features](features.md)
* A list of [Generic Substitutions](generic_substitutions.md)
* A set of [Object Modules](object_modules.md)