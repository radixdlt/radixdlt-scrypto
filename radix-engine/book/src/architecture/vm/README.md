# VM Layer

The VM Layer is responsible for passing control from the system to the application as well as
providing the application layer a Turing-complete computing environment and the interface to the
system layer.

Radix Engine currently supports two VM environments:
* A [Scrypto WASM VM](scrypto_vm.md) which exposes the system layer through WASM extern functions
* A [Native VM](native_vm.md) which directly compiles applications with Radix Engine in the host's environment

## Implementation

The VM Layer is implemented by defining the System Callback Object, which requires two callback
implementations:
1. `init` which is called on [transaction bootup](../../execution/transaction_lifecycle/bootup.md) to initialize the vm layer
2. `invoke` which is the entrypoint for any function or method invocation

On `invoke`, the VM layer determines the appropriate VM environment and then calls the associated
application layer logic.