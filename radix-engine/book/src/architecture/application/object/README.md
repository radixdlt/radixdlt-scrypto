# Object Model

Unlike Ethereum where the ledger state is a flat mapping between addresses and account states, Radix
Engine organizes its state into a forest of *objects*, each of which has a blueprint type. Child
objects are exclusively owned by its parent in the tree hierarchy. Each root object is assigned a
*global address*.

![](object_model.drawio.svg)