//! This module defines various types that the test runtime depends on and specifies what the
//! generic parameters are for them.

use crate::prelude::*;

pub type TestRuntimeVm<'g> = Vm<'g, DefaultWasmEngine, NoExtension>;
pub type TestRuntimeTrack<'g> = Track<'g, InMemorySubstateDatabase, SpreadPrefixKeyMapper>;
pub type TestRuntimeSystemConfig<'g> = SystemConfig<TestRuntimeVm<'g>>;
pub type TestRuntimeKernel<'g> = Kernel<'g, TestRuntimeSystemConfig<'g>, TestRuntimeTrack<'g>>;
pub type TestRuntimeSystemService<'g> = SystemService<'g, TestRuntimeKernel<'g>, TestRuntimeVm<'g>>;
