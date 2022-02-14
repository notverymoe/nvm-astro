/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::Component;

use super::resource_store::ResourceStore;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PortID {
    A = 0,
    B = 1,
    C = 2,
    D = 3
}

#[derive(Component)]
pub struct Ports([ResourceStore; 4]);

impl Ports {

    pub fn get(&self, value: PortID) -> &ResourceStore {
        unsafe{ self.0.get_unchecked(value as usize) }
    }

    pub fn get_mut(&mut self, value: PortID) -> &mut ResourceStore {
        unsafe{ self.0.get_unchecked_mut(value as usize) }
    }

}