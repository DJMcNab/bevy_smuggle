use std::ptr::NonNull;

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
    /// Which World this pointer is locked in
    world: WorldId,
    // Safety: The Mutability value must
    mutability: Mutability,
}

unsafe impl<T: 'static + Send + Sync> Send for StaticRef<T> {}
unsafe impl<T: 'static + Send + Sync> Sync for StaticRef<T> {}

impl<T: 'static> StaticRef<T> {
    pub(crate) fn new_shared(world: WorldId, reference: &T) -> Self {
        Self {
            ptr: reference.into(),
            world,
            mutability: Mutability::Shared,
        }
    }

    pub(crate) fn new_exclusive(world: WorldId, reference: &mut T) -> Self {
        Self {
            ptr: reference.into(),
            world,
            mutability: Mutability::Exclusive,
        }
    }
    pub(crate) unsafe fn read_shared_from<'a>(&self, world: WorldId) -> Option<&T> {
        if self.world != world {
            None
        } else {
            // Since this is a shared reference, either mutability is fine
            // Note that having &self means that only read_shared_from may be called whilst the reference lives
            Some(unsafe { self.ptr.as_ref() })
        }
    }
    pub(crate) unsafe fn read_exclusive_from(&mut self, world: WorldId) -> Option<&mut T> {
        if self.world != world || self.mutability != Mutability::Exclusive {
            None
        } else {
            Some(unsafe { self.ptr.as_mut() })
        }
    }
}
