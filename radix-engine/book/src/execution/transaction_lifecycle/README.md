# Transaction Lifecycle

Radix Engine is a transactional state machine which accepts a transaction and a given state and
outputs a state change and additional output.

```
radix_engine(State, Transaction) -> (StateChange, Output)
```

The state change can then be applied to the database to update it's state:

```
state_commit(State, StateChange) -> State
```

## Three Stages

There are three stages in the transaction lifecycle:
1. *Bootup*, which consists of initializing the layers of the stack
2. *Execution*, which is the execution of the application logic specified by the transaction
3. *Shutdown*, which consists of cleaning up each layer and creating the final StateChange and Output
