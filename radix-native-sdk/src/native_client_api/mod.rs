mod fields;

use radix_engine_interface::prelude::*;
use radix_common::prelude::*;
pub use fields::*;

pub struct NativeClientApi<'a, A: ClientApi<E>, E: Debug> {
    client_api: Rc<RefCell<&'a mut A>>,
    phantom_error: PhantomData<E>,
}

impl<'a, A: ClientApi<E>, E: Debug> NativeClientApi<'a, A, E> {
    pub fn new(client_api: &'a mut A) -> Self {
        Self {
            client_api: Rc::new(RefCell::new(client_api)),
            phantom_error: PhantomData,
        }
    }

    /// This is intended as an "escape-glass" fallback, but in future will be
    /// removed when it's no longer needed.
    pub fn raw_api(&self) -> RefMut<'_, &'a mut A> {
        self.client_api.borrow_mut()
    }
}

impl<'a, A: ClientApi<E>, E: Debug> From<&'a mut A> for NativeClientApi<'a, A, E> {
    fn from(value: &'a mut A) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy)]
pub enum SaveOnCloseMode {
    Save(SaveMode),
    DiscardChanges,
}

#[derive(Clone, Copy)]
pub enum SaveMode {
    OnlyIfChanged,
    SaveRegardless,
}

#[derive(Clone, Copy)]
pub enum AutoCloseMode {
    /// The kernel will automatically close it at the end of the frame
    NoCloseOnDrop,
    CloseOnDrop,
    ExpectManualCloseBeforeDrop,
}
