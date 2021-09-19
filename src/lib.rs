#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use bevy_ecs::{component::Component, world::World};
pub(crate) use static_ref::StaticRef;

mod static_ref;

mod params;
pub use params::{RefRes, RefResMut};

/// Temporarily store a shared reference within a [`World`], to be accessed using the
/// [`RefRes`] system parameter.
pub fn temporarily_store_shared_ref<T: Component, R>(
    world: &mut World,
    reference: &T,
    then: impl FnOnce(&mut World) -> R,
) -> R {
    let ref_ = unsafe { StaticRef::new_shared(world.id(), reference) };
    let guard = RemoveStaticRefOnDrop {
        world,
        t: PhantomData::<T>,
        disabler: ref_.disabler(),
    };
    guard.world.insert_resource(ref_);
    then(guard.world)
}

/// Temporarily store an exclusive reference within a [`World`], to be accessed using the
///  [`RefResMut`] or [`RefRes`] system parameters.
pub fn temporarily_store_exclusive_ref<T: Component, R>(
    world: &mut World,
    reference: &mut T,
    then: impl FnOnce(&mut World) -> R,
) -> R {
    let ref_ = unsafe { StaticRef::new_exclusive(world.id(), reference) };
    let guard = RemoveStaticRefOnDrop {
        world,
        t: PhantomData::<T>,
        disabler: ref_.disabler(),
    };
    guard.world.insert_resource(ref_);
    then(guard.world)
}

/// Remove a [`StaticRef`] from a [`World`] on [`Drop`]. This is used to clean up in
/// [`temporarily_store_exclusive_ref`] and [`temporarily_store_shared_ref`] functions
struct RemoveStaticRefOnDrop<'a, T: Component> {
    world: &'a mut World,
    t: PhantomData<T>,
    disabler: Arc<AtomicBool>,
}

impl<'a, T: Component> Drop for RemoveStaticRefOnDrop<'a, T> {
    fn drop(&mut self) {
        self.world.remove_resource::<StaticRef<T>>().or_else(|| {
            eprintln!(
                "Could not remove lifetime erased `&[mut] {}` from `World`",
                std::any::type_name::<T>()
            );
            self.disabler.store(true, Ordering::Relaxed);
            None
        });
    }
}
