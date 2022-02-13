/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::Component;
use std::cell::UnsafeCell;

use super::{ResourceID, ResourceStore};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortID {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

impl PortID {
    pub unsafe fn from_repr_unchecked(repr: u8) -> Self {
        core::mem::transmute(repr)
    }
}

#[derive(Component, Default)]
pub struct Ports {
    ports: [UnsafeCell<Port>; 4],
}

unsafe impl Sync for Ports {}

impl Ports {

    pub fn get(&self, id: PortID) -> &Port {
        unsafe{ &*self.get_unchecked(id) }
    }

    pub fn get_mut(&mut self, id: PortID) -> &mut Port {
        unsafe{ &mut *self.get_unchecked(id) }
    }

    pub unsafe fn get_unchecked(&self, id: PortID) -> *mut Port {
        self.ports.get_unchecked(id as usize).get()
    }
}

pub type PortCapacity = u16;

#[derive(Default, Clone, Copy, Debug)]
pub struct Port {
    store: ResourceStore<PortCapacity>,
}

impl Port {
    pub fn recv(&mut self, count: PortCapacity) -> Option<(ResourceID, PortCapacity)> {
        self.store.try_recv(count)
    }

    pub fn send(&mut self, resource: ResourceID, count: PortCapacity) -> Result<PortCapacity, ResourceID> {
        self.store.try_send(resource, count.min(self.remaining()))
    }

    pub fn remaining(&self) -> PortCapacity {
        u16::MAX - self.store.stored()
    }

    pub fn resource(&self) -> Option<ResourceID> {
        self.store.resource()
    }

    pub fn stored(&self) -> PortCapacity {
        self.store.stored()
    }
}
