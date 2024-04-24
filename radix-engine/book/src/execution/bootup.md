# Transaction Bootup

The initialization of a transaction execution consists of two steps:
1. Initialize Stack
2. Invoke Transaction Processor

![](bootup.drawio.png)

## Initialize Stack

Before a transaction is executed, initialization of the Kernel/System/VM stack occurs. During this
initialization phase, configuration is loaded from the database and the state of each layer is
initialized.

For example, during System initialization the system modules to run are decided on.
If we are executing in preview mode with auth disabled, the auth system module may be disabled.

The code for this can be found in [kernel.rs](../../src/kernel/kernel.rs) in the `Bootloader::execute`
method.

## Invoke Transaction Processor

Once the entire stack has been initialized along with the initial call frame, an invocation of a 
well-known blueprint, `TRANSACTION_PROCESSOR`, is made with the arguments specified in the transaction.
From this point forward, normal transaction execution occurs.
