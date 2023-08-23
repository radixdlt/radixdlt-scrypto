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
