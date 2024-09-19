//! This module defines various types that the test runtime depends on and specifies what the
//! generic parameters are for them.

use crate::prelude::*;

pub type TestVm<'g> = Vm<'g, DefaultWasmEngine, NoExtension>;
pub type TestTrack<'g, D> = Track<'g, D>;
pub type TestSystemConfig<'g> = System<TestVm<'g>>;
pub type TestKernel<'g, D> = Kernel<'g, TestSystemConfig<'g>, TestTrack<'g, D>>;
pub type TestSystemService<'g, D> = SystemService<'g, TestKernel<'g, D>>;
