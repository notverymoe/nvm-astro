/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::Component;
use std::cell::UnsafeCell;

use super::{ResourceID, ResourceStore};

pub type PortKey = u8;

#[derive(Component, Default)]
pub struct Ports {
    ports: Box<[UnsafeCell<Port>]>,
}

unsafe impl Sync for Ports {}

impl Ports {

    pub fn new(count: PortKey) -> Self {
        let count = count as usize;
        let mut ports = Vec::with_capacity(count);
        ports.resize_with(count, UnsafeCell::default);
        Self{ ports: ports.into_boxed_slice(), }
    }

    pub fn count(&self) -> PortKey {
        self.ports.len() as PortKey
    }

    pub fn get(&self, idx: PortKey) -> Option<&Port> {
        self.ports.get(idx as usize).map(|v| unsafe{ &*v.get() })
    }

    pub fn get_mut(&mut self, idx: PortKey) -> Option<&mut Port> {
        self.ports.get_mut(idx as usize).map(|v| v.get_mut())
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut_unchecked(&self, idx: PortKey) -> &mut Port {
        &mut *self.ports.get_unchecked(idx as usize).get()
    }
}

pub type PortCapacity = u16;

#[derive(Clone, Copy, Debug)]
pub struct Port {
    store: ResourceStore<PortCapacity>,
    pub capacity: PortCapacity,
}

impl Default for Port {
    fn default() -> Self {
        Self{
            store: ResourceStore::default(),
            capacity: PortCapacity::MAX,
        }
    }
}

impl Port {
    pub fn recv(&mut self, count: PortCapacity) -> Option<(ResourceID, PortCapacity)> {
        self.store.try_recv(count)
    }

    pub fn send(&mut self, resource: ResourceID, count: PortCapacity) -> Result<PortCapacity, ResourceID> {
        self.store.try_send(resource, count.min(self.remaining()))
    }

    pub fn remaining(&self) -> PortCapacity {
        self.capacity - self.store.stored()
    }

    pub fn resource(&self) -> Option<ResourceID> {
        self.store.resource()
    }

    pub fn stored(&self) -> PortCapacity {
        self.store.stored()
    }
}
