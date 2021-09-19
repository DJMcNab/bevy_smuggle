#![deny(unsafe_op_in_unsafe_fn)]

use std::ops::Deref;

use bevy_ecs::{
    component::Component,
    prelude::Res,
    system::{SystemParam, SystemParamFetch, SystemParamState},
};
use static_ref::StaticRef;

mod static_ref;

pub struct RefRes<'w, T: Component> {
    value: &'w T,
}

impl<'w, T: Component> SystemParam for RefRes<'w, T> {
    type Fetch = RefResState<T>;
}

pub struct RefResState<T: 'static + Send + Sync> {
    res: <Res<'static, StaticRef<T>> as SystemParam>::Fetch,
}

unsafe impl<'w, T: Component> SystemParamState for RefResState<T> {
    type Config = ();

    fn init(
        world: &mut bevy_ecs::prelude::World,
        system_meta: &mut bevy_ecs::system::SystemMeta,
        config: Self::Config,
    ) -> Self {
        Self {
            res: SystemParamState::init(world, system_meta, config),
        }
    }

    fn default_config() -> Self::Config {
        ()
    }
}

impl<'w, 's, T: Component> SystemParamFetch<'w, 's> for RefResState<T> {
    type Item = RefRes<'w, T>;

    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: &'w bevy_ecs::prelude::World,
        change_tick: u32,
    ) -> Self::Item {
        let res =
            unsafe { SystemParamFetch::get_param(&mut state.res, system_meta, world, change_tick) };
        let x = res.into_inner();
        RefRes {
            value: unsafe { x.read_shared_from(world.id()) }
                .expect("StaticRef<T> is only added to the correct World"),
        }
    }
}

impl<'w, T: Component> Deref for RefRes<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
