//! This module defines and implements the [`TestEnvironment`] struct.

use super::*;
use crate::prelude::*;

/// The environment that the tests are run against.
///
/// Each test environment has it's own instance of a [`SelfContainedRadixEngine`] which is exposed
/// through the [`ClientApi`] and which tests run against.
///
/// [`ClientApi`]: crate::prelude::ClientApi
pub struct TestEnvironment(pub(super) SelfContainedRadixEngine);

impl TestEnvironment {
    //=============
    // Invocations
    //=============

    /// Invokes a function on the provided blueprint and package with the given arguments.
    ///
    /// This method is a typed version of the [`ClientBlueprintApi::call_function`] which Scrypto
    /// encodes the arguments and Scrypto decodes the returns on behalf of the caller. This method
    /// assumes that the caller is correct about the argument and return types and panics if the
    /// encoding or decoding fails.
    ///
    /// # Arguments
    ///
    /// * `package_address`: [`PackageAddress`] - The address of the package that contains the
    /// blueprint.
    /// * `blueprint_name`: [`&str`] - The name of the blueprint.
    /// * `function_name`: [`&str`] - The nae of the function.
    /// * `args`: `&I` - The arguments to invoke the method with. This is a generic arguments that
    /// is fulfilled by any type that implements [`ScryptoEncode`].
    ///
    /// # Returns
    ///
    /// * [`Result<O, RuntimeError>`] - The returns from the method invocation. If the invocation
    /// was successful a [`Result::Ok`] is returned, otherwise a [`Result::Err`] is returned. The
    /// [`Result::Ok`] variant is a generic that's fulfilled by any type that implements
    /// [`ScryptoDecode`].
    ///
    /// # Panics
    ///
    /// This method panics in the following two cases:
    ///
    /// * Through an unwrap when calling [`scrypto_encode`] on the method arguments. Please consult
    /// the SBOR documentation on more information on why SBOR encoding may fail.
    /// * Through an unwrap when calling [`scrypto_decode`] on the returns. This panics if the type
    /// could be decoded as the desired output type.
    pub fn call_function_typed<I, O>(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: &I,
    ) -> Result<O, RuntimeError>
    where
        I: ScryptoEncode,
        O: ScryptoDecode,
    {
        let args = scrypto_encode(args).expect("Scrypto encoding of args failed");
        self.call_function(package_address, blueprint_name, function_name, args)
            .map(|rtn| scrypto_decode(&rtn).expect("Scrypto decoding of returns failed"))
    }

    /// Invokes a method on the main module of a node with the provided typed arguments.
    ///
    /// This method is a typed version of the [`ClientObjectApi::call_method`] which Scrypto encodes
    /// the arguments and Scrypto decodes the returns on behalf of the caller. This method assumes
    /// that the caller is correct about the argument and return types and panics if the encoding or
    /// decoding fails.
    ///
    /// # Arguments
    ///
    /// * `node_id`: `T` - The node to invoke the method on. This is a generic argument that's
    /// fulfilled by any type that implements [`Into<NodeId>`], thus, any address type can be used.
    /// * `method_name`: [`&str`] - The name of the method to invoke.
    /// * `args`: `&I` - The arguments to invoke the method with. This is a generic arguments that
    /// is fulfilled by any type that implements [`ScryptoEncode`].
    ///
    /// # Returns
    ///
    /// * [`Result<O, RuntimeError>`] - The returns from the method invocation. If the invocation
    /// was successful a [`Result::Ok`] is returned, otherwise a [`Result::Err`] is returned. The
    /// [`Result::Ok`] variant is a generic that's fulfilled by any type that implements
    /// [`ScryptoDecode`].
    ///
    /// # Panics
    ///
    /// This method panics in the following two cases:
    ///
    /// * Through an unwrap when calling [`scrypto_encode`] on the method arguments. Please consult
    /// the SBOR documentation on more information on why SBOR encoding may fail.
    /// * Through an unwrap when calling [`scrypto_decode`] on the returns. This panics if the type
    /// could be decoded as the desired output type.
    pub fn call_method_typed<N, I, O>(
        &mut self,
        node_id: N,
        method_name: &str,
        args: &I,
    ) -> Result<O, RuntimeError>
    where
        N: Into<NodeId>,
        I: ScryptoEncode,
        O: ScryptoDecode,
    {
        let args = scrypto_encode(args).expect("Scrypto encoding of args failed");
        self.call_method(&node_id.into(), method_name, args)
            .map(|rtn| scrypto_decode(&rtn).expect("Scrypto decoding of returns failed"))
    }

    /// Invokes a method on the main module of a node with the provided typed arguments.
    ///
    /// This method is a typed version of the [`ClientObjectApi::call_method`] which Scrypto encodes
    /// the arguments and Scrypto decodes the returns on behalf of the caller. This method assumes
    /// that the caller is correct about the argument and return types and panics if the encoding or
    /// decoding fails.
    ///
    /// # Arguments
    ///
    /// * `node_id`: `T` - The node to invoke the method on. This is a generic argument that's
    /// fulfilled by any type that implements [`Into<NodeId>`], thus, any address type can be used.
    /// * `method_name`: [`&str`] - The name of the method to invoke.
    /// * `args`: `&I` - The arguments to invoke the method with. This is a generic arguments that
    /// is fulfilled by any type that implements [`ScryptoEncode`].
    ///
    /// # Returns
    ///
    /// * [`Result<O, RuntimeError>`] - The returns from the method invocation. If the invocation
    /// was successful a [`Result::Ok`] is returned, otherwise a [`Result::Err`] is returned. The
    /// [`Result::Ok`] variant is a generic that's fulfilled by any type that implements
    /// [`ScryptoDecode`].
    ///
    /// # Panics
    ///
    /// This method panics in the following two cases:
    ///
    /// * Through an unwrap when calling [`scrypto_encode`] on the method arguments. Please consult
    /// the SBOR documentation on more information on why SBOR encoding may fail.
    /// * Through an unwrap when calling [`scrypto_decode`] on the returns. This panics if the type
    /// could be decoded as the desired output type.
    pub fn call_direct_access_method_typed<N, I, O>(
        &mut self,
        node_id: N,
        method_name: &str,
        args: &I,
    ) -> Result<O, RuntimeError>
    where
        N: Into<NodeId>,
        I: ScryptoEncode,
        O: ScryptoDecode,
    {
        let args = scrypto_encode(args).expect("Scrypto encoding of args failed");
        self.call_direct_access_method(&node_id.into(), method_name, args)
            .map(|rtn| scrypto_decode(&rtn).expect("Scrypto decoding of returns failed"))
    }

    /// Invokes a method on a module of a node with the provided typed arguments.
    ///
    /// This method is a typed version of the [`ClientObjectApi::call_method`] which Scrypto encodes
    /// the arguments and Scrypto decodes the returns on behalf of the caller. This method assumes
    /// that the caller is correct about the argument and return types and panics if the encoding or
    /// decoding fails.
    ///
    /// # Arguments
    ///
    /// * `node_id`: `T` - The node to invoke the method on. This is a generic argument that's
    /// fulfilled by any type that implements [`Into<NodeId>`], thus, any address type can be used.
    /// * `module`: [`ModuleId`] - The module id.
    /// * `method_name`: [`&str`] - The name of the method to invoke.
    /// * `args`: `&I` - The arguments to invoke the method with. This is a generic arguments that
    /// is fulfilled by any type that implements [`ScryptoEncode`].
    ///
    /// # Returns
    ///
    /// * [`Result<O, RuntimeError>`] - The returns from the method invocation. If the invocation
    /// was successful a [`Result::Ok`] is returned, otherwise a [`Result::Err`] is returned. The
    /// [`Result::Ok`] variant is a generic that's fulfilled by any type that implements
    /// [`ScryptoDecode`].
    ///
    /// # Panics
    ///
    /// This method panics in the following two cases:
    ///
    /// * Through an unwrap when calling [`scrypto_encode`] on the method arguments. Please consult
    /// the SBOR documentation on more information on why SBOR encoding may fail.
    /// * Through an unwrap when calling [`scrypto_decode`] on the returns. This panics if the type
    /// could be decoded as the desired output type.
    pub fn call_module_method_typed<N, I, O>(
        &mut self,
        node_id: N,
        module: ModuleId,
        method_name: &str,
        args: &I,
    ) -> Result<O, RuntimeError>
    where
        N: Into<NodeId>,
        I: ScryptoEncode,
        O: ScryptoDecode,
    {
        let args = scrypto_encode(args).expect("Scrypto encoding of args failed");
        self.call_module_method(&node_id.into(), module, method_name, args)
            .map(|rtn| scrypto_decode(&rtn).expect("Scrypto decoding of returns failed"))
    }

    //====================================
    // Manipulation of the Kernel Modules
    //====================================

    /// Enables the kernel trace kernel module of the Radix Engine.
    pub fn enable_kernel_trace_module(&mut self) {
        self.enable_module(EnabledModules::KERNEL_TRACE)
    }

    /// Enables the limits kernel module of the Radix Engine.
    pub fn enable_limits_module(&mut self) {
        self.enable_module(EnabledModules::LIMITS)
    }

    /// Enables the costing kernel module of the Radix Engine.
    pub fn enable_costing_module(&mut self) {
        self.enable_module(EnabledModules::COSTING)
    }

    /// Enables the auth kernel module of the Radix Engine.
    pub fn enable_auth_module(&mut self) {
        self.enable_module(EnabledModules::AUTH)
    }

    /// Enables the transaction runtime kernel module of the Radix Engine.
    pub fn enable_transaction_runtime_module(&mut self) {
        self.enable_module(EnabledModules::TRANSACTION_RUNTIME)
    }

    /// Enables the execution trace kernel module of the Radix Engine.
    pub fn enable_execution_trace_module(&mut self) {
        self.enable_module(EnabledModules::EXECUTION_TRACE)
    }

    /// Disables the kernel trace kernel module of the Radix Engine.
    pub fn disable_kernel_trace_module(&mut self) {
        self.disable_module(EnabledModules::KERNEL_TRACE)
    }

    /// Disables the limits kernel module of the Radix Engine.
    pub fn disable_limits_module(&mut self) {
        self.disable_module(EnabledModules::LIMITS)
    }

    /// Disables the costing kernel module of the Radix Engine.
    pub fn disable_costing_module(&mut self) {
        self.disable_module(EnabledModules::COSTING)
    }

    /// Disables the auth kernel module of the Radix Engine.
    pub fn disable_auth_module(&mut self) {
        self.disable_module(EnabledModules::AUTH)
    }

    /// Disables the transaction runtime kernel module of the Radix Engine.
    pub fn disable_transaction_runtime_module(&mut self) {
        self.disable_module(EnabledModules::TRANSACTION_RUNTIME)
    }

    /// Disables the execution trace kernel module of the Radix Engine.
    pub fn disable_execution_trace_module(&mut self) {
        self.disable_module(EnabledModules::EXECUTION_TRACE)
    }

    /// Calls the passed `callback` with the kernel trace kernel module enabled and then resets the
    /// state of the kernel modules.
    pub fn with_kernel_trace_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_kernel_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the limits kernel module enabled and then resets the state
    /// of the kernel modules.
    pub fn with_limits_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_limits_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the costing kernel module enabled and then resets the state
    /// of the kernel modules.
    pub fn with_costing_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_costing_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the auth kernel module enabled and then resets the state of
    /// the kernel modules.
    pub fn with_auth_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_auth_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the transaction runtime kernel module enabled and then
    /// resets the state of the kernel modules.
    pub fn with_transaction_runtime_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_transaction_runtime_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the execution trace kernel module enabled and then resets
    /// the state of the kernel modules.
    pub fn with_execution_trace_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_execution_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the kernel trace kernel module disabled and then resets the
    /// state of the kernel modules.
    pub fn with_kernel_trace_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_kernel_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the limits kernel module disabled and then resets the state
    /// of the kernel modules.
    pub fn with_limits_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_limits_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the costing kernel module disabled and then resets the
    /// state of the kernel modules.
    pub fn with_costing_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_costing_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the auth kernel module disabled and then resets the state
    /// of the kernel modules.
    pub fn with_auth_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_auth_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the transaction runtime kernel module disabled and then
    /// resets the state of the kernel modules.
    pub fn with_transaction_runtime_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_transaction_runtime_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the execution trace kernel module disabled and then resets
    /// the state of the kernel modules.
    pub fn with_execution_trace_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_execution_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Returns the bit flags representing the currently enabled kernel modules.
    pub fn enabled_modules(&self) -> EnabledModules {
        self.0
            .with_kernel(|kernel| kernel.kernel_callback().modules.enabled_modules)
    }

    /// Sets the bit flags representing the enabled kernel modules.
    pub fn set_enabled_modules(&mut self, enabled_modules: EnabledModules) {
        self.0.with_kernel_mut(|kernel| {
            kernel.kernel_callback_mut().modules.enabled_modules = enabled_modules
        })
    }

    /// Enables specific kernel module(s).
    pub fn enable_module(&mut self, module: EnabledModules) {
        self.0.with_kernel_mut(|kernel| {
            kernel.kernel_callback_mut().modules.enabled_modules |= module
        })
    }

    /// Disables specific kernel module(s).
    pub fn disable_module(&mut self, module: EnabledModules) {
        self.0.with_kernel_mut(|kernel| {
            kernel.kernel_callback_mut().modules.enabled_modules &= !module
        })
    }
}
