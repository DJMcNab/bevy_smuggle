#![deny(unsafe_op_in_unsafe_fn)]

use std::marker::PhantomData;

use bevy_ecs::{component::Component, world::World};
pub(crate) use static_ref::StaticRef;

mod static_ref;

mod params;
pub use params::{RefRes, RefResMut};

pub fn temporarily_store_shared_ref<T: Component, R>(
    world: &mut World,
    reference: &T,
    then: impl FnOnce(&mut World) -> R,
) -> R {
    let guard = RemoveStaticRefOnDrop {
        world,
        t: PhantomData::<T>,
    };
    guard
        .world
        .insert_resource(unsafe { StaticRef::new_shared(guard.world.id(), reference) });
    then(guard.world)
}

pub fn temporarily_store_exclusive_ref<T: Component, R>(
    world: &mut World,
    reference: &mut T,
    then: impl FnOnce(&mut World) -> R,
) -> R {
    let guard = RemoveStaticRefOnDrop {
        world,
        t: PhantomData::<T>,
    };
    guard
        .world
        .insert_resource(unsafe { StaticRef::new_exclusive(guard.world.id(), reference) });
    then(guard.world)
}

struct RemoveStaticRefOnDrop<'a, T: Component> {
    world: &'a mut World,
    t: PhantomData<T>,
}

impl<'a, T: Component> Drop for RemoveStaticRefOnDrop<'a, T> {
    fn drop(&mut self) {
        self.world
            .remove_resource::<StaticRef<T>>()
            .unwrap_or_else(|| {
                panic!(
                    "Could not remove lifetime erased `&[mut] {}`",
                    std::any::type_name::<T>()
                )
            });
    }
}
