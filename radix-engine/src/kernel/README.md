# Kernel

The primary goal of the Kernel in the Radix Engine is to maintain Ownership and Reference invariants. More specifically,
the Kernel enforces that there is one and only one owner for any Node object in the system and that every reference to a Node
object is a valid reference (no dangling pointers or NULL pointers).