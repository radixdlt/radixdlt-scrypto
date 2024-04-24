# Transaction Lifecycle

Radix Engine is a state machine which follows:
```
RadixEngine(State, Transaction) -> StateChange
```

There are three stages in the transaction lifecycle:
1. Bootup
2. Execution
3. Shutdown

Bootup consists of setting up the state of the layered stack.

Execution is running application logic.

Shutdown is cleaning up the layered stack and creating the final StateChange receipt.