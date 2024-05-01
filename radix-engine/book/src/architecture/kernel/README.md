# Kernel Layer

The kernel layer implements Node/Partition/Substate abstraction on top of a key-value database as well
as maintains a call frame and transaction state updates, which are to be subsequently committed to the
database at the end of the transaction.