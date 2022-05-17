/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::mem::MaybeUninit;

use bevy::prelude::{Component, Entity};

use super::ResourceID;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortID {
    A = 0,
    B = 1,
    C = 2,
    D = 3
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PortRecv(pub Entity, pub PortID);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PortSend(pub Entity, pub PortID);

#[derive(Component, Default)]
pub struct Ports([ResourceStore; 4]);

impl Ports {

    pub fn get(&self, value: PortID) -> &ResourceStore {
        unsafe{ self.0.get_unchecked(value as usize) }
    }

    pub fn get_mut(&mut self, value: PortID) -> &mut ResourceStore {
        unsafe{ self.0.get_unchecked_mut(value as usize) }
    }

}

pub struct ResourceStore {
    resource: MaybeUninit<ResourceID>,
    count:    u16,
}

impl Default for ResourceStore {
    fn default() -> Self {
        Self { resource: MaybeUninit::uninit(), count: 0 }
    }
}

impl ResourceStore {

    pub fn count(&self) -> u16 {
        self.count
    }

    pub fn set(&mut self, resource: ResourceID, count: u16) {
        self.resource = MaybeUninit::new(resource);
        self.count    = count;
    }

    pub fn get(&self) -> Option<(ResourceID, u16)> {
        match self.count {
            0 => None,
            _ => Some((unsafe{ self.resource.assume_init() }, self.count))
        }
    }

    pub fn get_or(&self, resource: ResourceID) -> (ResourceID, u16) {
        match self.count {
            0 => (resource, 0),
            _ => (unsafe{ self.resource.assume_init() }, self.count),
        }
    }
    
    pub fn clear(&mut self) {
        self.count = 0;
    }

}