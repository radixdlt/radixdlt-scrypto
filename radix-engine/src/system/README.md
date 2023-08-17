# System

The System Layer acts as a rather fat layer between the application and the kernel. It provides
a layer through which all applications must interact with and thus allows for both application standardization
(e.g. Authorization) and global services (e.g. Type Checking and Memory Protection).

## System Modules

The system uses pluggable modules to implement application standardization.

| Module                                            | Description                                                                                                                                                                          |
|---------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [Auth](system_modules/auth)                       | Manages Authorization of Blueprint function and method calls.                                                                                                                        |
| [Costing](system_modules/costing)                 | Manages Runtime costing (aka "gas") as well as Royalties. Stops execution of a transaction if all reserves have been used.                                                           |
| [Limits](system_modules/limits)                   | Similar to costing except rather than deal with a costing reserve simply tracks explicit resource useage during a transaction. Stops execution of a transaction if any limit is hit. |
| [Runtime](system_modules/transaction_runtime)     | Stores transaction runtime state which is not stored on ledger but may be used by the application layer, the transaction id, for example.                                            |
| [Execution Trace](system_modules/execution_trace) | Keeps track of various information during runtime to return in the receipt.                                                                                                          |
| [Kernel Trace](system_modules/kernel_trace)       | Logs various information as it occurs during runtime.                                                                                                                                |

Note that concepts such as Type Checking and Memory Protection are not currently implemented as modules as they
are "deeper" constructs in the System Layer (at least in the present moment).