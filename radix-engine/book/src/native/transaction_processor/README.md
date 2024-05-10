# Transaction Processor

The transaction processor is the initial application layer call frame made during
the [transaction boot process](../../execution/transaction_lifecycle/bootup.md) and executes a transaction manifest which is encoded in
a transaction.

It consists of a blueprint with a single `run` function.