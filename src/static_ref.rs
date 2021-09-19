use std::{
    ptr::NonNull,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use bevy_ecs::world::WorldId;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mutability {
    Exclusive,
    Shared,
}

/// Stores a reference to data outside a bevy_ecs world
pub(crate) struct StaticRef<T: 'static> {
    /// The data behind the pointer
    ptr: NonNull<T>,
    /// Which World this pointer is linked to
    world: WorldId,
    // Safety: The Mutability value must be correct
    mutability: Mutability,
    /// Whether this [`StaticRef`] has been externally disabled
    /// This disabling happens when the lifetime of `ptr` would end
    /// Note that both this and `world` must be checked, since
    /// without that check the lifetime could end partway through a system
    disabler: Arc<AtomicBool>,
}

unsafe impl<T: 'static + Send + Sync> Send for StaticRef<T> {}
unsafe impl<T: 'static + Send + Sync> Sync for StaticRef<T> {}

impl<T: 'static> StaticRef<T> {
    pub(crate) unsafe fn new_shared(world: WorldId, reference: &T) -> Self {
        Self {
            ptr: reference.into(),
            world,
            mutability: Mutability::Shared,
            disabler: Default::default(),
        }
    }

    pub(crate) unsafe fn new_exclusive(world: WorldId, reference: &mut T) -> Self {
        Self {
            ptr: reference.into(),
            world,
            mutability: Mutability::Exclusive,
            disabler: Default::default(),
        }
    }

    pub(crate) fn disabler(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.disabler)
    }

    fn check_invariants(&self, world: WorldId) -> Option<()> {
        (self.world == world && !self.disabler.load(Ordering::Relaxed)).then(|| ())
    }

    pub(crate) unsafe fn read_shared_from<'a>(&self, world: WorldId) -> Option<&T> {
        self.check_invariants(world)?;
        // Since this is a shared reference, either mutability is fine
        // Note that having &self means that only read_shared_from may be called whilst the reference lives
        Some(unsafe { self.ptr.as_ref() })
    }
    pub(crate) unsafe fn read_exclusive_from(&mut self, world: WorldId) -> Option<&mut T> {
        self.check_invariants(world)?;
        if self.mutability != Mutability::Exclusive {
            None
        } else {
            Some(unsafe { self.ptr.as_mut() })
        }
    }
}
