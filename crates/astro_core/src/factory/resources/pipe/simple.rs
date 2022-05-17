/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::Component;

use super::{ResourceID, Pipe};

#[derive(Component)]
pub struct PipeSimple(PacketBuffer);

impl PipeSimple {

    pub fn new(length: u32) ->  Self {
        Self(PacketBuffer::new(length))
    }

    pub fn get_packet_position(&self, factory_tick: u32, i: u32) -> usize {
        if i >= self.0.len() { panic!("Attempt to index out of bounds") }
        let capacity = self.0.capacity() as u32;
        let distance_from_start = factory_tick - self.0.get(i).unwrap().0;
        distance_from_start.min((capacity - i - 1) as u32).min(capacity-1) as usize
    }
}

impl Pipe for PipeSimple {

    unsafe fn enqueue_unchecked(&mut self, tick: u32, resource: ResourceID) {
        self.0.push(tick, resource);
    }

    unsafe fn consume_unchecked(&mut self) {
        self.0.pop();
    }

    unsafe fn get_unchecked(&self) -> ResourceID {
        self.0.peek_front().unwrap().1
    }

    fn is_full(&self) -> bool {
        self.0.is_full()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_ready_to_consume(&self, tick: u32) -> bool {
        !self.0.is_empty() && tick - self.0.peek_front().unwrap().0 >= self.0.capacity() as u32
    }

    fn resolve(&self, factory_tick: u32) -> Box<[Option<ResourceID>]> {
        let mut result = vec![None; self.0.capacity() as usize].into_boxed_slice();
        for i in 0..self.0.len() {
            let idx = self.get_packet_position(factory_tick, i);
            result[idx] = self.0.get(i).map(|v| v.1);
        }
        result
    }

}

pub struct PacketBuffer {
    data: Box<[(u32, Option<ResourceID>)]>,
    head: u32,
    tail: u32,
}

impl PacketBuffer {

    pub fn new(capacity: u32) -> Self {
        Self{
            data: vec![(0, None); capacity as usize].into_boxed_slice(),
            head: 0,
            tail: 0
        }
    }

    pub fn capacity(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn len(&self) -> u32 {
        let (head, tail, capacity) = (self.head as usize, self.tail as usize, self.data.len());
        (if tail == capacity { 
            capacity
        } else if tail < head {
            (capacity + tail) - head
        } else {
            tail - head
        }) as u32
    }

    pub fn is_empty(&self) -> bool {
        self.tail == self.head
    }

    pub fn is_full(&self) -> bool {
        self.tail == self.data.len() as u32
    }

    pub fn push(&mut self, tick: u32, value: ResourceID) {
        if self.is_full() { panic!("Buffer is full"); }
        self.data[self.tail as usize] = (tick, Some(value));
        self.inc_tail();
    }

    pub fn peek_front(&self) -> Option<(u32, ResourceID)> {
        let result = self.data[self.head as usize];
        result.1.map(|v| (result.0, v))
    }

    pub fn peek_back(&self) -> Option<(u32, ResourceID)> {
        let result = self.data[self.get_last_idx()];
        result.1.map(|v| (result.0, v))
    }

    pub fn pop(&mut self) {
        if self.is_empty() { panic!("Buffer is empty"); }
        self.data[self.head as usize].1 = None;
        self.inc_head();
    }

    pub fn get(&self, idx: u32) -> Option<(u32, ResourceID)> {
        if idx >= self.len() { return None; }
        let (tick, resource) = self.data[(self.head as usize + idx as usize) % self.data.len()];
        Some((tick, resource.unwrap()))
    }

    fn inc_head(&mut self) {
        let head_old = self.head;
        self.head = wrap_inc(self.head, self.capacity());
        if self.tail == self.data.len() as u32 {
            self.tail = head_old;
        }
    }

    fn inc_tail(&mut self) {
        self.tail = wrap_inc(self.tail, self.capacity());
        if self.tail == self.head {
            self.tail = self.data.len() as u32;
        }
    }

    fn get_last_idx(&self) -> usize {
        if self.tail == self.capacity() {
            wrap_dec(self.head, self.capacity()) as usize
        } else {
            wrap_dec(self.tail, self.capacity()) as usize
        }
    }

}

#[inline] fn wrap_inc(value: u32, max: u32) -> u32 {
    if value >= max - 1 {
        0
    } else {
        value + 1
    }
}

#[inline] fn wrap_dec(value: u32, max: u32) -> u32 {
    if value == 0 {
        max - 1
    } else {
        value - 1
    }
}