use std::ops::{Deref, DerefMut};

use crate::StaticRef;
use bevy_ecs::{
    component::Component,
    prelude::{Res, ResMut},
    system::{SystemParam, SystemParamFetch, SystemParamState},
};

pub struct RefRes<'w, T: Component> {
    value: &'w T,
}

impl<'w, T: Component> RefRes<'w, T> {
    pub fn into_inner(&self) -> &'w T {
        self.value
    }
}

impl<'w, T: Component> Deref for RefRes<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct RefResMut<'w, T: Component> {
    value: &'w mut T,
}

impl<'w, T: Component> RefResMut<'w, T> {
    pub fn into_inner(self) -> &'w mut T {
        self.value
    }
}

impl<'w, T: Component> Deref for RefResMut<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'w, T: Component> DerefMut for RefResMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'w, T: Component> SystemParam for RefResMut<'w, T> {
    type Fetch = RefResMutState<T>;
}

pub struct RefResMutState<T: 'static + Send + Sync> {
    res: <ResMut<'static, StaticRef<T>> as SystemParam>::Fetch,
}

unsafe impl<'w, T: Component> SystemParamState for RefResMutState<T> {
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

impl<'w, 's, T: Component> SystemParamFetch<'w, 's> for RefResMutState<T> {
    type Item = RefResMut<'w, T>;

    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: &'w bevy_ecs::prelude::World,
        change_tick: u32,
    ) -> Self::Item {
        let res =
            unsafe { SystemParamFetch::get_param(&mut state.res, system_meta, world, change_tick) };
        let x = res.into_inner();
        RefResMut {
            value: unsafe { x.read_exclusive_from(world.id()) }.expect(
                "StaticRef<T> should only be present in the correct `World` with the correct\
mutability and during its lifetime",
            ),
        }
    }
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
