# Scrypto Wasm VM

On `invoke` of a method which executes in a Scrypto Wasm VM, the VM layer loads the WASM code
associated with the invocation and creates a new WASM instance with a fresh heap and stack.
The exported function associated with the invocation is then called with the
invocation arguments.

Extern functions are mapped to subset of the system layer's api which the application can call.