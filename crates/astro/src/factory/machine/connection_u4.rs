/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::cell::UnsafeCell;

use bevy::prelude::Component;

use super::{ResourceID, ConnectionQueue};

#[derive(Component)]
pub struct ConnectionU4 {
    length:   u8,
    capacity: u8,
    queue:   UnsafeCell<[Option<ResourceID>; 16]>,
}

unsafe impl Sync for ConnectionU4 {}
unsafe impl Send for ConnectionU4 {}

impl ConnectionU4 {

    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0 && capacity <= 16);
        Self{
            capacity: capacity as u8,
            length:   0,
            queue:   [None; 16].into(),
        }
    }

    unsafe fn do_consume(&self) {
        let queue = (*self.queue.get()).as_mut_ptr();
        core::ptr::copy(queue.offset(1), queue, 15);
        *queue.offset(15) = None;
    }

    fn head(&self) -> Option<ResourceID> {
        self.get(0)
    }

    fn get(&self, idx: usize) -> Option<ResourceID> {
        unsafe{(*self.queue.get())[idx]}
    }

}

impl ConnectionQueue for ConnectionU4 {

    unsafe fn enqueue_unchecked(&mut self, _tick: u32, resource: ResourceID) {
        self.queue.get_mut()[(self.capacity as usize)-1] = Some(resource);
        self.length += 1;
    }

    unsafe fn consume_unchecked(&mut self) {
        self.do_consume();
        self.length -= 1;
    }

    unsafe fn get_unchecked(&self) -> ResourceID {
        self.head().unwrap_unchecked()
    }

    fn is_full(&self) -> bool {
        self.length >= self.capacity // TODO test
    }

    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn is_ready_to_consume(&self, _tick_factory: u32) -> bool {
        if self.head().is_none() {
            unsafe{ self.do_consume(); }
            false
        } else {
            true
        }
    }

    fn resolve(&self, _factory_tick: u32) -> Box<[Option<ResourceID>]> {
        let capacity = self.capacity as usize;
        let mut result = vec![None; capacity].into_boxed_slice();
        for i in 0..capacity as usize {
            result[capacity - (i+1)] = self.get(i);
        }
        result
    }

}