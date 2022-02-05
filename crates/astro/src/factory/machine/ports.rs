/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::Component;
use nvm_bevyutil::sync::{SyncCell, SyncMutRef};

use super::{ResourceID, ResourceStore};

pub type PortKey = u8;

#[derive(Component, Default)]
pub struct Ports {
    ports: Box<[SyncCell<Port>]>,
}

impl Ports {

    pub fn new(count: PortKey) -> Self {
        let count = count as usize;
        let mut ports = Vec::with_capacity(count);
        ports.resize_with(count, SyncCell::<Port>::default);
        Self{ ports: ports.into_boxed_slice(), }
    }

    pub fn count(&self) -> PortKey {
        self.ports.len() as PortKey
    }

    pub fn get(&self, idx: PortKey) -> Option<SyncMutRef<Port>> {
        self.ports.get(idx as usize).map(|v| v.borrow_mut())
    }

    pub fn get_mut(&mut self, idx: PortKey) -> Option<&mut Port> {
        self.ports.get_mut(idx as usize).map(|v| v.get_mut())
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Port> {
        self.ports.iter_mut().map(|v| v.get_mut())
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
        self.capacity.saturating_sub(self.store.stored())
    }

    pub fn resource(&self) -> Option<ResourceID> {
        self.store.resource()
    }

    pub fn stored(&self) -> PortCapacity {
        self.store.stored()
    }
}
