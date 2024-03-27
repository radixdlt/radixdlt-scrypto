use radix_engine_interface::prelude::*;
use radix_common::prelude::*;
use radix_rust::rust::ops::*;
use super::*;

pub trait FieldDefinition {
    const FIELD_INDEX: u8;
    type Payload: ScryptoEncode + ScryptoDecode;
    type Content: ScryptoEncode + ScryptoDecode;
}

/// A temporary "type" for a cleaner API, will be compiled out.
pub struct FieldSource<'n, F: FieldDefinition, N: NativeFieldOpenApi<E>, E: Debug> {
    native_api: &'n N,
    actor_state_handle: ActorStateHandle,
    phantom_definition: PhantomData<F>,
    phantom_error: PhantomData<E>,
}

impl<'n, F: FieldDefinition, N: NativeFieldOpenApi<E>, E: Debug> FieldSource<'n, F, N, E> {
    pub fn new(native_api: &'n N, actor_state_handle: ActorStateHandle) -> Self {
        Self {
            native_api,
            actor_state_handle,
            phantom_definition: PhantomData,
            phantom_error: PhantomData,
        }
    }

    /// Immediately reads the field, and returns an immutable reference to its content.
    pub fn open_readonly<'f>(&self) -> Result<FieldContentRef<'f, F, N::InternalFieldApi, E>, E> {
        let field_ref = self.open_advanced(LockFlags::read_only(), AutoCloseMode::CloseOnDrop)?;
        field_ref.into_read_content()
    }

    /// Immediately reads the field, and returns a mutable reference to its content.
    /// When dropped, the field is written only if the value has been changed.
    pub fn open_readwrite<'f>(&self) -> Result<FieldContentMut<'f, F, N::InternalFieldApi, E>, E> {
        let field_ref = self.open_advanced(LockFlags::MUTABLE, AutoCloseMode::CloseOnDrop)?;
        field_ref.into_mutable_read_content(SaveOnCloseMode::Save(SaveMode::OnlyIfChanged))
    }

    /// Opens the handle to the field, but does not read it yet.
    /// Can be converted into readonly or readwrite content references, with more complicated behaviour.
    pub fn open_advanced<'f>(&self, lock_flags: LockFlags, close_mode: AutoCloseMode) -> Result<FieldRef<'f, F, N::InternalFieldApi, E>, E> {
        self.native_api.open_field(self.actor_state_handle, lock_flags, close_mode)
    }
}

pub trait NativeFieldOpenApi<E: Debug> {
    type InternalFieldApi: InternalNativeFieldApi<E>;

    /// Opens a wrapped handle to the field, but does not read it yet.
    fn open_field<'f, F: FieldDefinition>(
        &self,
        actor_state_handle: ActorStateHandle,
        lock_flags: LockFlags,
        close_mode: AutoCloseMode,
    ) -> Result<FieldRef<'f, F, Self::InternalFieldApi, E>, E>;
}

impl <'a, A: ClientApi<E>, E: Debug> NativeFieldOpenApi<E> for NativeClientApi<'a, A, E> {
    type InternalFieldApi = Rc<RefCell<&'a mut A>>;
    
    fn open_field<'f, F: FieldDefinition>(
        &self,
        actor_state_handle: ActorStateHandle,
        lock_flags: LockFlags,
        auto_close_mode: AutoCloseMode
    ) -> Result<FieldRef<'f, F, Self::InternalFieldApi, E>, E> {
        FieldRef::open(
            self.client_api.clone(),
            actor_state_handle,
            F::FIELD_INDEX,
            lock_flags,
            auto_close_mode
        )
    }
}

pub trait InternalNativeFieldApi<E: Debug> {
    fn open(&self, state_handle: ActorStateHandle, field: FieldIndex, flags: LockFlags) -> Result<FieldHandle, E>;

    fn read(&self, handle: FieldHandle) -> Result<Vec<u8>, E>;

    fn write(&self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), E>;

    fn lock(&self, handle: FieldHandle) -> Result<(), E>;

    fn close(&self, handle: FieldHandle) -> Result<(), E>;
}

impl<'a, E: Debug, A: ClientActorApi<E> + ClientFieldApi<E>> InternalNativeFieldApi<E> for Rc<RefCell<&'a mut A>> {
    fn open(&self, state_handle: ActorStateHandle, field: FieldIndex, flags: LockFlags) -> Result<FieldHandle, E> {
        self.borrow_mut().actor_open_field(state_handle, field, flags)
    }

    fn read(&self, handle: FieldHandle) -> Result<Vec<u8>, E> {
        self.borrow_mut().field_read(handle)
    }

    fn write(&self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), E> {
        self.borrow_mut().field_write(handle, buffer)
    }

    fn lock(&self, handle: FieldHandle) -> Result<(), E> {
        self.borrow_mut().field_lock(handle)
    }

    fn close(&self, handle: FieldHandle) -> Result<(), E> {
        self.borrow_mut().field_close(handle)
    }
}

/// An immutable reference to a field's value.
/// Upon creation, the current value of the field is read.
pub struct FieldContentRef<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> {
    #[allow(dead_code)] // The field ref is needed to keep the field open
    field_ref: FieldRef<'a, F, A, E>,
    content: F::Content,
}

impl<'a, F: FieldDefinition<Content = V>, V: fmt::Display, A: InternalNativeFieldApi<E>, E: Debug> fmt::Display for FieldContentRef<'a, F, A, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content.fmt(f)
    }
}

impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> FieldContentRef<'a, F, A, E> {
    pub fn new(field_ref: FieldRef<'a, F, A, E>) -> Result<Self, E> {
        let buf = field_ref.read()?;
        let content = scrypto_decode(&buf).map_err(|e| e).unwrap();
        Ok(Self {
            field_ref,
            content,
        })
    }

    /// Closes the field handle, and returns the content.
    pub fn close_field_keeping_content(self) -> F::Content {
        // Implicitly self.field_ref is dropped, closing the field
        self.content
    }

    pub fn close_field(self) {}
}

impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> Deref for FieldContentRef<'a, F, A, E> {
    type Target = F::Content;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}


/// A mutable reference to a field's value.
/// Upon creation, the current value of the field is read.
pub struct FieldContentMut<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> {
    field_ref: Option<FieldRef<'a, F, A, E>>,
    save_on_close_mode: SaveOnCloseMode,
    is_dirty: bool,
    original_raw_content: Vec<u8>,
    content: F::Content,
}

impl<'a, F: FieldDefinition<Content = V>, V: fmt::Display, A: InternalNativeFieldApi<E>, E: Debug> fmt::Display for FieldContentMut<'a, F, A, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content.fmt(f)
    }
}

impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> FieldContentMut<'a, F, A, E> {
    pub fn new(field_ref: FieldRef<'a, F, A, E>, save_on_close_mode: SaveOnCloseMode) -> Result<Self, E>
    {
        let original_raw_value = field_ref.read()?;
        let value = scrypto_decode(&original_raw_value).map_err(|e| e).unwrap();
        Ok(Self {
            field_ref: Some(field_ref),
            save_on_close_mode,
            is_dirty: false,
            original_raw_content: original_raw_value,
            content: value,
        })
    }

    /// PRECONDITION: Cannot be used mid-closing after being closed
    fn open_field_ref(&self) -> &FieldRef<'a, F, A, E> {
        self.field_ref.as_ref().unwrap()
    }

    pub fn manual_save_field(&self, save_mode: SaveMode) -> Result<bool, E> {
        let write_value = match save_mode {
            SaveMode::OnlyIfChanged => {
                if !self.is_dirty {
                    None
                } else {
                    let value = scrypto_encode(&self.content).unwrap();
                    if value != self.original_raw_content {
                        Some(value)
                    } else {
                        None
                    }
                }
            },
            SaveMode::SaveRegardless => {
                let value = scrypto_encode(&self.content).unwrap();
                Some(value)
            },
        };
        match write_value {
            Some(value) => self.open_field_ref().write(value).map(|()| true),
            None => Ok(false),
        }
    }

    pub fn lock_field(&mut self) -> Result<(), E> {
        self.open_field_ref().lock()
    }

    pub fn save_and_close_field(self, save_mode: SaveMode) -> Result<(), E> {
        self.manual_save_field(save_mode)?;
        self.close_field_without_saving()
    }

    pub fn close_field_without_saving(mut self) -> Result<(), E> {
        let field_ref = mem::replace(&mut self.field_ref, None).unwrap();
        field_ref.close()?;
        self.save_on_close_mode = SaveOnCloseMode::DiscardChanges;
        Ok(())
    }
}

impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> Drop for FieldContentMut<'a, F, A, E> {
    fn drop(&mut self) {
        // If we're currently panicking, then this means the native call frame will be aborted:
        // * The panic will be caught by the native component panic handler in `native_vm.rs`
        // * The transaction will then terminate and roll back any changes.
        //
        // In this case, saving and closing is irrelevant, so we can just skip them.
        // This also ensures that we avoid crashing here due to panicking during a panic!
        // See https://doc.rust-lang.org/std/ops/trait.Drop.html#panics
        //
        // Even if in future we support try/catch, the kernel would need to handle rolling back
        // saves and open handles, so this is still the correct behavior.
        #[cfg(feature = "std")]
        if std::thread::panicking() {
            return;
        }

        // Note, as this wraps the field_ref, this drop method runs first, therefore we save before the field is closed.
        match self.save_on_close_mode {
            SaveOnCloseMode::Save(save_mode) => {
                self.manual_save_field(save_mode).unwrap();
            },
            SaveOnCloseMode::DiscardChanges => {}
        }
    }
}

impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> Deref for FieldContentMut<'a, F, A, E> {
    type Target = F::Content;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> DerefMut for FieldContentMut<'a, F, A, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_dirty = true;
        &mut self.content
    }
}

/// An open reference to a field. Upon creation, no field is read.
pub struct FieldRef<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> {
    handle: FieldHandle,
    api: A,
    auto_close_mode: AutoCloseMode,
    phantom_field_handle_lifetime: PhantomData<&'a ()>,
    phantom_field_definition: PhantomData<F>,
    phantom_error: PhantomData<E>,
}

impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> FieldRef<'a, F, A, E> {
    pub fn open(api: A, actor_state_handle: ActorStateHandle, field_index: FieldIndex, lock_flags: LockFlags, auto_close_mode: AutoCloseMode) -> Result<Self, E> {
        let handle = api.open(actor_state_handle, field_index, lock_flags)?;
        Ok(Self {
            handle,
            api,
            auto_close_mode,
            phantom_field_handle_lifetime: PhantomData,
            phantom_field_definition: PhantomData,
            phantom_error: PhantomData,
        })
    }

    pub fn read(&self) -> Result<Vec<u8>, E> {
        self.api.read(self.handle)
    }

    pub fn write(&self, buffer: Vec<u8>) -> Result<(), E> {
        self.api.write(self.handle, buffer)
    }

    pub fn lock(&self) -> Result<(), E> {
        self.api.lock(self.handle)
    }

    pub fn close(mut self) -> Result<(), E> {
        self.auto_close_mode = AutoCloseMode::NoCloseOnDrop;
        self.api.close(self.handle)
    }

    pub fn into_read_content(self) -> Result<FieldContentRef<'a, F, A, E>, E> {
        FieldContentRef::new(self)
    }

    /// PRECONDITION: To use this method, please ensure that the field was originally opened with the correct flags
    pub fn into_mutable_read_content(self, save_on_close_mode: SaveOnCloseMode) -> Result<FieldContentMut<'a, F, A, E>, E> {
        FieldContentMut::new(self, save_on_close_mode)
    }
}

//===========================================
// !!!! DISCUSSION POINT FOR PEER REVIEW !!!!
//===========================================
// This isn't actually correct until we change the error model in native components.
//
// This is because native components have to propagate kernel errors transparently,
// without doing anything else => BUT these drop implementations add side effects
// around saving/closing things after a failure.
//
// Therefore, either we need to change the error first handling first (and encourage
// native components to panic on error)
//
// OR push this out with AutoCloseMode::NoCloseOnDrop / SaveOnCloseMode::DiscardChanges
impl<'a, F: FieldDefinition, A: InternalNativeFieldApi<E>, E: Debug> Drop for FieldRef<'a, F, A, E> {
    fn drop(&mut self) {
        // If we're currently panicking, then this means the native call frame will be aborted:
        // * The panic will be caught by the native component panic handler in `native_vm.rs`
        // * The transaction will then terminate and roll back any changes.
        //
        // In this case, closing is irrelevant, so we can just skip it.
        // This also ensures that we avoid crashing here due to panicking during a panic!
        // See https://doc.rust-lang.org/std/ops/trait.Drop.html#panics
        //
        // Even if in future we support try/catch, the kernel would need to handle rolling back
        // open handles, so this is still the correct behavior.
        #[cfg(feature = "std")]
        if std::thread::panicking() {
            return;
        }

        match self.auto_close_mode {
            AutoCloseMode::NoCloseOnDrop => {},
            AutoCloseMode::CloseOnDrop => {
                self.api.close(self.handle).unwrap();
            },
            AutoCloseMode::ExpectManualCloseBeforeDrop => {
                // Manually closing replaces the behaviour with AutoCloseMode::NoCloseOnDrop and then drops self
                // Therefore this case when not manually closed
                panic!("Expected field to be manually closed, but it was not. This safety-check caused a panic.");
            },
        }

    }
}