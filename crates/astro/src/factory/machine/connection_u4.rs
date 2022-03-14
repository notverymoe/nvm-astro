/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::{cell::{UnsafeCell, Cell}};

use bevy::prelude::Component;

use super::{ResourceID, ConnectionQueue};

#[derive(Component)]
pub struct ConnectionU4 {
    head: Cell<u8>,
    tail: u8,
    capacity:  u8,
    index: Cell<u64>,
    queue: UnsafeCell<[Option<ResourceID>; 16]>,
}

unsafe impl Sync for ConnectionU4 {}
unsafe impl Send for ConnectionU4 {}

impl ConnectionU4 {

    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0 && capacity <= 16);
        Self{
            head: 0.into(),
            tail: 0,
            capacity: capacity as u8,
            index:    0.into(),
            queue: [None; 16].into(),
        }
    }

    unsafe fn do_consume(&self) {
        self.index.set(self.index.get() >> 4);

        let queue = (*self.queue.get()).as_mut_ptr();
        core::ptr::copy(queue.add(1), queue, 15);
        *queue.add(15) = None;
    }

    unsafe fn do_advance(&self) {
        let tail = self.tail;
        if tail == 0 { 
            return; 
        }

        let head  = self.head.get();
        let index = self.index.get();
        let mask  = (0xFFFF_FFFF_FFFF_FFFFu64.checked_shl(4*head as u32).unwrap_or(0))
                 & !(0xFFFF_FFFF_FFFF_FFFFu64.checked_shl(4*tail as u32).unwrap_or(0));

        self.index.set((index & !mask) | ((index & mask) - (0x1111_1111_1111_1111 & mask)));
        if (self.index.get() >> (4*head)) & 0xF == 0 {
            self.head.set(head+1);
        }
    }

    fn waiting_to_send(&self) -> bool {
        (self.tail > 0) && (self.index.get() & 0xF == 0)
    }

    fn get(&self, idx: usize) -> Option<ResourceID> {
        unsafe{(*self.queue.get())[idx]}
    }

}

impl ConnectionQueue for ConnectionU4 {

    unsafe fn enqueue_unchecked(&mut self, _tick: u32, resource: ResourceID) {
        self.queue.get_mut()[self.tail as usize] = Some(resource);
        self.index.set(self.index.get() | (((self.capacity - 1) as u64) << (4*self.tail)));
        self.tail += 1;
    }

    unsafe fn consume_unchecked(&mut self) {
        self.do_consume();
        self.head.set(0);
        self.tail -= 1;
    }

    unsafe fn get_unchecked(&self) -> ResourceID {
        self.get(0).unwrap_unchecked()
    }

    fn is_full(&self) -> bool {
        self.tail >= self.capacity
    }

    fn is_empty(&self) -> bool {
        self.tail == 0
    }

    fn is_ready_to_consume(&self, _tick_factory: u32) -> bool {
        let result = self.waiting_to_send();
        unsafe { self.do_advance() }
        result
    }

    fn resolve(&self, _factory_tick: u32) -> Box<[Option<ResourceID>]> {
        let capacity = self.capacity as usize;
        let tail = self.tail as usize;
        let mut result = vec![None; capacity].into_boxed_slice();
        let index = self.index.get() as usize;
        for i in 0..tail {
            let pos = ((index >> (i*4)) & 0xF).max(i);
            let idx = capacity - (1 + pos);
            result[idx] = self.get(i);
        }
        result
    }

}