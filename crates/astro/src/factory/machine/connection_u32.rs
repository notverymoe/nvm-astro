/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Component};

use super::{ResourceID, PacketBuffer, ConnectionQueue};

pub type ConnectionDuration = u16;

#[derive(Component)]
pub struct ConnectionU32(PacketBuffer);

impl ConnectionU32 {

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

impl ConnectionQueue for ConnectionU32 {

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
        !self.0.is_empty() && tick - self.0.peek_front().unwrap().0 > self.0.capacity() as u32
    }

    fn resolve(&self, factory_tick: u32) -> Box<[Option<ResourceID>]> {
        let mut result = vec![None; self.0.capacity() as usize].into_boxed_slice();
        for i in 0..self.0.len() {
            let idx = self.get_packet_position(factory_tick, i);
            if result[idx].is_some() {
                println!("Oops");
            }
            result[idx] = self.0.get(i).map(|v| v.1);
        }
        result
    }

}