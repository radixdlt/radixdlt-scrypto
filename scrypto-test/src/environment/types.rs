//! This module defines various types that the test runtime depends on and specifies what the
//! generic parameters are for them.

use crate::prelude::*;

pub type TestVm<'g> = Vm<'g, DefaultWasmEngine, NoExtension>;
pub type TestTrack<'g> = Track<'g, InMemorySubstateDatabase, SpreadPrefixKeyMapper>;
pub type TestSystemConfig<'g> = SystemConfig<TestVm<'g>>;
pub type TestKernel<'g> = Kernel<'g, TestSystemConfig<'g>, TestTrack<'g>>;
pub type TestSystemService<'g> = SystemService<'g, TestKernel<'g>, TestVm<'g>>;
