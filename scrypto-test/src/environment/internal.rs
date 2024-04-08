//! This module contains the internal representation of the testing environment which is a self
//! contained Radix Engine implemented as a self-referencing struct.

use super::*;
use core::ops::*;
use radix_common::*;
use radix_engine::kernel::id_allocator::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_interface::*;
use radix_substate_store_interface::interface::*;

// TODO: I would like to remove the reliance on `CommittableSubstateDatabase` and to instead commit
//       everything to the track. As in, nothing ever gets committed to the database. Even the
//       initial bootstrapping should be done in this way. This mainly comes from a desire to use
//       the node's database with scrypto-test, and it does not implement that trait.

/// The implementation of a self-contained Radix Engine.
///
/// This is a self-contained Radix Engine that uses the [`ouroboros`] crate for self-referencing to
/// allow the entire Radix Engine stack to be stored in a single struct where some members reference
/// one another. As an example: the [`Track`] references the substate database stored in the same
/// object as it.
#[ouroboros::self_referencing(no_doc)]
pub(super) struct EncapsulatedRadixEngine<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    pub(super) substate_db: D,
    pub(super) scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    pub(super) native_vm: NativeVm<NoExtension>,
    pub(super) id_allocator: IdAllocator,

    #[borrows(substate_db)]
    #[covariant]
    pub(super) track: TestTrack<'this, D>,

    #[borrows(scrypto_vm)]
    #[covariant]
    pub(super) system_config: TestSystemConfig<'this>,

    #[borrows(mut system_config, mut track, mut id_allocator)]
    #[not_covariant]
    pub(super) kernel: TestKernel<'this, D>,
}

impl<D> EncapsulatedRadixEngine<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    pub(super) fn create(
        substate_db: D,
        scrypto_vm: ScryptoVm<DefaultWasmEngine>,
        native_vm: NativeVm<NoExtension>,
        id_allocator: IdAllocator,
        track_builder: impl FnOnce(&D) -> TestTrack<'_, D>,
        system_config_builder: impl FnOnce(&ScryptoVm<DefaultWasmEngine>) -> TestSystemConfig<'_>,
        kernel_builder: impl for<'a> FnOnce(
            &'a mut TestSystemConfig<'a>,
            &'a mut TestTrack<'a, D>,
            &'a mut IdAllocator,
        ) -> TestKernel<'a, D>,
    ) -> Self {
        EncapsulatedRadixEngineBuilder {
            substate_db,
            scrypto_vm,
            native_vm,
            id_allocator,
            track_builder,
            system_config_builder,
            kernel_builder,
        }
        .build()
    }
}
