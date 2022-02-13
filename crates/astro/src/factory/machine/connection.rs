/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::collections::VecDeque;

use bevy::prelude::{Entity, Query, Component};

use super::{ResourceID, Ports, Port, PortID};
use nvm_bevyutil::try_unwrap_option;

pub type ConnectionDuration = u16;

#[derive(Component)]
pub struct Connection {
    length: ConnectionDuration,
    queue: VecDeque<ConnectionDuration>,
    resource_ids: VecDeque<ResourceID>,

    head_idx: u16,
    tail_distance: ConnectionDuration,
}

impl Connection {

    pub fn new(length: ConnectionDuration) -> Self {
        Self{
            length,
            queue:        VecDeque::with_capacity(length as usize),
            resource_ids: VecDeque::with_capacity(length as usize),
            head_idx: 0,
            tail_distance: length,
        }
    }

    pub fn update(&mut self) {
        if self.head_idx as usize >= self.queue.len() {
            return;
        }

        self.queue[self.head_idx as usize] -= 1;
        self.tail_distance += 1;
        while self.can_advance_head() { self.head_idx += 1; }
    }

    fn can_advance_head(&mut self) -> bool {
        let head = self.head_idx as usize;
        if head >= self.queue.len() { return false; }
        if self.queue[head] > 1 { return false; }
        true
    }

    pub fn try_insert(&mut self, resource: ResourceID) -> bool {

        if self.can_recieve() {
            if self.queue.len() > self.length as usize {
                let mut a = Vec::new();
                self.resolve(&mut a);
                println!("{:?} {:?}", self.queue, a);
            }
            self.queue.push_back(self.tail_distance);
            self.resource_ids.push_back(resource);
            self.tail_distance = 0;
            true
        } else {
            false
        }
    }

    pub fn resolve(&self, destination: &mut Vec<i32>) {
        let mut counter: u16 = 0;
        destination.extend(self.queue.iter().map(|v| { counter += v; self.length as i32 - counter as i32  }));
    } 

    pub fn can_recieve(&self) -> bool {
        self.tail_distance != 0
    }
}

impl Connection {
    pub fn pop_send(&mut self) {
        if self.queue.is_empty() { return; }

        self.head_idx = 0;
        self.queue.pop_front();
        self.resource_ids.pop_front();
        if !self.queue.is_empty() { 
            self.queue[0] += 1; 
        } else {
            self.tail_distance = self.length;
        }
    }

    pub fn can_send(&self) -> bool {
        let head = self.head_idx as usize;
        head < self.queue.len() && (head != 0 || self.queue[0] == 0)
    }

    pub fn peek_send(&self) -> Option<ResourceID> {
        self.resource_ids.get(0).copied()
    }
}


#[derive(Component)]
pub struct ConnectionIO {
    from: Entity, 
    to:   Entity,
    ports: u8,
}

impl ConnectionIO {

    pub fn new(
        from: Entity, 
        from_port: PortID, 
        to: Entity, 
        to_port: PortID,
    ) -> Self {
        assert!(from != to || from_port != to_port, "Machine cannot connect to the same slot on itself");

        let from_port = from_port as u8;
        let   to_port =   to_port as u8;
        Self{from, to, ports: from_port | (to_port << 2)}
    }

    pub unsafe fn try_unchecked_send(&self, connection: &mut Connection, q: &Query<&Ports>) {
        if !connection.can_send() { return; }
        let send = connection.peek_send().unwrap();
        if let Some(sent) = Self::get_port_unchecked(q, self.from, self.port_from()).and_then(|v| unsafe{&mut *v}.send(send, 1).ok()) {
            if sent > 0 { connection.pop_send(); }
        }
    }

    pub unsafe fn try_unchecked_recv(&self, connection: &mut Connection, q: &Query<&Ports>) {
        if !connection.can_recieve() { return; }
        let (resource, count) = try_unwrap_option!(Self::get_port_unchecked(q, self.to, self.port_to()).and_then(|v| unsafe{&mut *v}.recv(1)));
        if count > 0 { connection.try_insert(resource); }
    }

    pub fn try_send(&self, connection: &mut Connection, q: &mut Query<&mut Ports>) {
        if !connection.can_send() { return; }
        let send = connection.peek_send().unwrap();
        if let Some(sent) = Self::get_port_mut(q, self.from, self.port_from()).and_then(|v| v.send(send, 1).ok()) {
            if sent > 0 { connection.pop_send(); }
        }
    }

    pub fn try_recv(&self, connection: &mut Connection, q: &mut Query<&mut Ports>) {
        if !connection.can_recieve() { return; }
        let (resource, count) = try_unwrap_option!(Self::get_port_mut(q, self.to, self.port_to()).and_then(|v| v.recv(1)));
        if count > 0 { connection.try_insert(resource); }
    }


    pub fn port_from(&self) -> PortID {
        unsafe{ PortID::from_repr_unchecked(self.ports & 0x03) }
    }

    pub fn port_to(&self) -> PortID {
        unsafe{ PortID::from_repr_unchecked((self.ports >> 2) & 0x03) }
    }

    fn get_port_mut<'a>(q: &'a mut Query<&mut Ports>, entity: Entity, port: PortID) -> Option<&'a mut Port> {
        q.get_mut(entity).ok().map(|v| v.into_inner().get_mut(port))
    }

    unsafe fn get_port_unchecked(q: &Query<&Ports>, entity: Entity, port: PortID) -> Option<*mut Port> {
        q.get_unchecked(entity).map(|v| v.get_unchecked(port)).ok()
    }
}