# Native VM

On `invoke` of a function/method which executes natively, the function must have been compiled with
Radix Engine and is just called directly. Because all applications using the Native VM are trusted,
Native VM applications are not isolated from the Radix Engine and share the same memory space.

The full system layer API is also exposed to Native VM applications.
