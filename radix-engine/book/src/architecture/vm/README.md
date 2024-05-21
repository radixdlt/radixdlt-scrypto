# VM Layer

The VM Layer is responsible for providing the application layer a Turing-complete computing
environment and the interface to the system layer.

## Implementation

Radix Engine currently supports two VM environments:
* A Scrypto WASM VM which exposes the system layer through WASM extern functions
* A Native VM which directly compiles applications with Radix Engine in the host's environment

The VM Layer is implemented by defining the System Callback Object, which requires two callback
implementations:
1. `init` which is called on [transaction bootup](../../execution/transaction_lifecycle/bootup.md) to initialize the vm layer
2. `invoke` which is the entrypoint for any function or method invocation

### Invoke Callback

On `invoke`, the VM layer loads the code and the vm environment associated with the invocation.

If the vm environment is WASM, a new WASM instance is created with a fresh heap and stack.
The exported function associated with the invocation is then called with the invocation arguments.
Extern functions expose a subset of the system layer's api which the application can call.

If the vm environment is Native, the function must have been compiled with Radix Engine and the
function is just called directly. Because all applications using the Native VM are trusted, Native VM
applications are not isolated from the Radix Engine and share the same memory space. The full system
layer API is also exposed to Native VM applications.
