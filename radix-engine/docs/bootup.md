# Transaction Bootup

The initialization of a transaction execution consists of two steps:
1. Initialization of Kernel/System/VM
2. Invocation of the Transaction Processor

![](bootup.drawio.png)

## Initialization

Before a transaction can be executed, initialization of Kernel/System/VM must occur. Several things occur
at the point:
1. Kernel creates the initial call frame
2. Loading of System Layer configuration such as Fee Configuration from Ledger
3. Loading of VM Configuration from Ledger

This is also the point where the system modules to run are decided on. For example, if we are executing
in preview mode with auth disabled, we may have the auth system module disabled.

The substates read for configuration load are not accessible to any other packages as it is part of the special
Transaction Processor Component of which there is only one.