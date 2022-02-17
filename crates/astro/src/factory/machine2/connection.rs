/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Query, Component, Res};

use crate::factory::machine::{ResourceID, RingBuffer, Ports, ConnectionPortRecv, ConnectionPortSend, ResourceIDInnerType};

pub struct FactoryTick(pub u32);

#[derive(Default, Clone, Copy)]
pub struct Packet(u32, ResourceIDInnerType);

#[derive(Component)]
pub struct Connection(RingBuffer<Packet>);

impl Connection {

    pub fn new(length: u16) ->  Self {
        Self(RingBuffer::new(length))
    }

    pub fn can_consume(&self, tick: u32) -> bool {
        !self.0.is_empty() && self.0.front().0 >= tick
    }

    pub fn can_insert(&self) -> bool {
        !self.0.is_full()
    }

    pub fn insert(&mut self, tick: u32, resource: ResourceID) {
        self.0.push_back(Packet(tick + self.0.capacity() as u32, resource.into_inner()));
    }

    pub fn consume(&mut self) {
        self.0.pop_front();
    }

    pub fn peek(&self) -> (u32, ResourceID) {
        let front = self.0.front();
        (front.0, unsafe{ResourceID::from_inner_unchecked(front.1)})
    }
}

pub fn connection_recv(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut Connection, &ConnectionPortRecv)>,
    mut ports: Query<&mut Ports>
) {
    for (mut connection, ports_recv) in connections.iter_mut() {
        if !connection.can_insert() { continue; }
        if let Ok(mut ports) = ports.get_mut(ports_recv.0) {
            if let Some((resource, count)) = ports.get(ports_recv.1).get() {
                ports.get_mut(ports_recv.1).set(resource, count-1);
                connection.insert(tick.0, resource);
            }
        }
    }
}

pub fn connection_send(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut Connection, &ConnectionPortSend)>,
    mut ports: Query<&mut Ports>
) {
    for (mut connection, ports_send) in connections.iter_mut() {
        if !connection.can_consume(tick.0) {  continue; }
        if let Ok(mut ports) = ports.get_mut(ports_send.0) {
            let (resource, count) =  ports.get(ports_send.1).get_or(connection.peek().1);
            if resource != connection.peek().1 { continue; }
            ports.get_mut(ports_send.1).set(resource, count+1);
            connection.consume();
        }
    }
}

// TODO move front inside struct?
// TODO port on connection?
// TODO port impls