
use bevy::prelude::Component;

use super::{ResourceID, ConnectionQueue};

#[derive(Component, Clone, Copy)]
pub struct ConnectionU4 {
    capacity:  u8,
    length:    u8,
    queue:     u64,
    resources: [Option<ResourceID>; 16]
}

impl ConnectionU4 {

    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0 && capacity <= 16);
        Self{
            capacity:  capacity as u8,
            length:    0,
            queue:     0,
            resources: [None; 16],
        }
    }

    pub fn capacity(&self) -> u8 {
        self.capacity
    }

    pub fn len(&self) -> u8 {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

}


impl ConnectionQueue for ConnectionU4 {
    unsafe fn enqueue_unchecked(&mut self, tick: u32, resource: ResourceID) {
        self.resources[self.length as usize] = Some(resource);
        self.queue |= (tick as u64 & 0x0F) << (self.length as u64 * 4);
        self.length += 1;
    }

    unsafe fn consume_unchecked(&mut self) {
        self.queue >>= 4;
        core::ptr::copy(self.resources[1..].as_ptr(), self.resources.as_mut_ptr(), 15);
        self.length -= 1;
    }

    unsafe fn get_unchecked(&self) -> ResourceID {
        self.resources.get_unchecked(0).unwrap_unchecked()
    }

    fn is_full(&self) -> bool {
        self.length > 0
    }

    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn is_ready_to_consume(&self, tick_factory: u32) -> bool {
        if self.is_empty() { return false }
        let due_tick = self.capacity as u32 + resolve_tick(tick_factory, self.queue as u32) - 1;
        due_tick <= tick_factory
    }

    fn resolve(&self, factory_tick: u32) -> Box<[Option<ResourceID>]> {

        let mut copy = *self;
        let capacity = self.capacity as usize;

        let mut result = vec![None; capacity].into_boxed_slice();
        for i in 0..capacity as usize {
            if copy.is_empty() { break; }
            if copy.is_ready_to_consume(factory_tick + i as u32) {
                result[capacity - (i + 1)] = Some(unsafe{ copy.get_unchecked() });
                unsafe{ copy.consume_unchecked(); }
            }
        }
        result
    }

}

fn resolve_tick(tick_factory: u32, queue_tick: u32) -> u32 {
    ((tick_factory - 1) & 0xFFFF_FFF0) | (queue_tick & 0x0F)
}