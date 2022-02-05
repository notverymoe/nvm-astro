/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::collections::VecDeque;

use bevy::prelude::{Entity, Query, Component};
use nvm_bevyutil::{sync::SyncMutRef, try_unwrap_option};

use super::{PortKey, ResourceID, Ports, Port, PortCapacity};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPacket {
    pub time:     u32,
    pub resource: ResourceID,
    pub stored:   PortCapacity,
}

#[derive(Component, Debug)]
pub struct Connection {
    from: (Entity, PortKey),
    to:   (Entity, PortKey),

    transfer_count: PortCapacity,
    transfer_time:  u32,
    packets: VecDeque<ConnectionPacket>,
}

impl Connection {

    pub fn new(
        transfer_count: PortCapacity, 
        transfer_time: u32, 
        from: (Entity, PortKey), 
        to: (Entity, PortKey)
    ) -> Self {
        assert!(from.0 != to.0 || from.1 != to.1, "Machine cannot connect to the same slot on itself");
        Self{
            from,
            to,
            transfer_count,
            transfer_time,
            packets: VecDeque::with_capacity((transfer_time as usize)+1), // Should never re-allocate
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ConnectionPacket> {
        self.packets.iter()
    }

    pub fn len(&self) -> usize {
        self.packets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    pub fn can_recieve(&self) -> bool {
        self.packets.is_empty() || self.packets[self.packets.len()-1].time != 0
    }
}


impl Connection {
    pub fn tick(&mut self) {
        let mut tick_lim = self.transfer_time;
        for packet in self.packets.iter_mut() {
            if packet.time + 1 < tick_lim { packet.time += 1; }
            tick_lim = packet.time;
        }
    }

    pub fn try_send(&mut self, q: &Query<&Ports>) {
        let last = try_unwrap_option!(self.get_send_packet());
        let took = try_unwrap_option!(self.lookup_to(q).and_then(|mut v| v.send(last.resource, last.stored).ok()));
        if last.stored <= took { self.pop(); }
    }

    pub fn try_recv(&mut self, q: &Query<&Ports>) {
        if !self.can_recieve() { return; }
        let (kind, stored) = try_unwrap_option!(self.lookup_from(q).and_then(|mut v| v.recv(self.transfer_count)));
        self.push(ConnectionPacket{resource: kind, stored, time: 0});
    }

    fn lookup_from<'a>(&self, q: &'a Query<&Ports>) -> Option<SyncMutRef<'a, Port>> {
        q.get(self.from.0).ok().and_then(|v| v.get(self.from.1))
    }

    fn lookup_to<'a>(&self, q: &'a Query<&Ports>) -> Option<SyncMutRef<'a, Port>> {
        q.get(self.to.0).ok().and_then(|v| v.get(self.to.1))
    }
}


impl Connection {

    fn push(&mut self, packet: ConnectionPacket) {
        self.packets.push_back(packet);
    }

    fn pop(&mut self) {
        self.packets.pop_front();
    }

    fn get_send_packet(&self) -> Option<ConnectionPacket> {
        match !self.packets.is_empty() && (self.packets[0].time + 1 >= self.transfer_time) {
            true  => Some(self.packets[0]),
            false => None,
        }
    }
}