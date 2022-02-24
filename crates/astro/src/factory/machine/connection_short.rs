/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::Component;

use super::{ResourceID, ResourceIDInnerType, ConnectionQueue};

#[derive(Component)]
pub struct ConnectionShort {
    /* -1 */ indicies:   u8,
    /* -8 */ queue:      u64,
    /* 32 */ resources: [ResourceIDInnerType; 16],
    /*----*/
    /* 41 */ // This is bad for that cache line thing right? 64/41 => ~1.3? Ew.
}

impl ConnectionShort {

    pub const fn new(capacity: u8) -> Self {
        assert!(capacity <= 16);
        Self {
            indicies: capacity << 4,
            queue: 0,
            resources: [0; 16],
        }
    }

    fn capacity(&self) -> u8 {
        self.indicies >> 4
    }

    fn len(&self) -> u8 {
        self.indicies & 0x0F
    }

}

impl ConnectionQueue for ConnectionShort {
    unsafe fn enqueue_unchecked(&mut self, tick: u32, resource: ResourceID) {
        self.resources[self.len() as usize] = resource.into_inner();
        self.queue |= (tick as u64 & 0x0F) << (self.len() as u64 * 4);
        self.indicies += 1;
    }

    unsafe fn consume_unchecked(&mut self) {
        self.indicies -= 1;
        self.queue >>= 4;
        core::ptr::copy(self.resources[1..].as_ptr(), self.resources.as_mut_ptr(), 15);
    }

    unsafe fn get_unchecked(&self) -> ResourceID {
        ResourceID::from_inner_unchecked(*self.resources.get_unchecked(0))
    }

    fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_ready_to_consume(&self, tick_factory: u32) -> bool {
        !self.is_empty() && tick_factory >= (((tick_factory - 1) & 0xFFFF_FFF7) | (self.queue as u32 & 0x0F))
    }

}
