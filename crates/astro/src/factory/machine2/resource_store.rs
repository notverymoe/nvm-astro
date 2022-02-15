/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::mem::MaybeUninit;

use super::resource::ResourceID;

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
    
    pub fn clear(&mut self) {
        self.count = 0;
    }

}