/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Component};

use super::{ResourceID, RingBuffer, ResourceIDInnerType, ConnectionQueue};

pub type ConnectionDuration = u16;

#[repr(C, align(8))]
#[derive(Default, Clone, Copy)]
pub struct Packet(u32, ResourceIDInnerType);

#[derive(Component)]
pub struct ConnectionU16(RingBuffer<Packet>);

impl ConnectionU16 {

    pub fn new(length: u16) ->  Self {
        Self(RingBuffer::new(length))
    }
}

impl ConnectionQueue for ConnectionU16 {

    unsafe fn enqueue_unchecked(&mut self, tick: u32, resource: ResourceID) {
        self.0.push_back(Packet(tick + self.0.capacity() as u32, resource.into_inner()));
    }

    unsafe fn consume_unchecked(&mut self) {
        self.0.pop_front();
    }

    unsafe fn get_unchecked(&self) -> ResourceID {
        ResourceID::from_inner_unchecked(self.0.front().1)
    }

    fn is_full(&self) -> bool {
        self.0.len() >= self.0.capacity()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_ready_to_consume(&self, tick: u32) -> bool {
        !self.0.is_empty() && self.0.front().0 < tick
    }

    fn resolve(&self, factory_tick: u32) -> Box<[Option<ResourceID>]> {
        let mut result = vec![None; self.0.capacity().into()].into_boxed_slice();
        for i in 0..self.0.len() {
            let position = self.0.get(i).0 - factory_tick;
            result[position as usize] = ResourceID::try_from_inner(self.0.get(i).1);
        }
        result
    }

}