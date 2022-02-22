/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Query, Component, Res, Mut, Without};

use crate::factory::FactoryTick;

use super::{ResourceID, RingBuffer, Ports, ResourceIDInnerType, ConnectionPortRecv, ConnectionPortSend};

pub type ConnectionDuration = u16;

#[repr(C, align(8))]
#[derive(Default, Clone, Copy)]
pub struct Packet(u32, ResourceIDInnerType);

#[derive(Component)]
pub struct ConnectionLong(RingBuffer<Packet>);

impl ConnectionLong {

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

pub fn connection_send_recv(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut ConnectionLong, &ConnectionPortRecv, &ConnectionPortSend)>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_recv, ports_send) in connections.iter_mut() {
        do_connection_send(tick, &mut connection, ports_send, &mut ports);
        do_connection_recv(tick, &mut connection, ports_recv, &mut ports);
    }
}

pub fn connection_recv(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut ConnectionLong, &ConnectionPortRecv), Without<ConnectionPortSend>>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_recv) in connections.iter_mut() {
        do_connection_recv(tick, &mut connection, ports_recv, &mut ports)
    }
}

pub fn connection_send(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut ConnectionLong, &ConnectionPortSend), Without<ConnectionPortRecv>>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_send) in connections.iter_mut() {
        do_connection_send(tick, &mut connection, ports_send, &mut ports)
    }
}

fn do_connection_recv(
    tick: u32,
    connection: &mut Mut<ConnectionLong>,
    ports_recv: &ConnectionPortRecv,
    ports: &mut Query<&mut Ports>
) {
    if !connection.can_insert() { return; }
    if let Ok(mut ports) = ports.get_mut(ports_recv.0) {
        if let Some((resource, count)) = ports.get(ports_recv.1).get() {
            ports.get_mut(ports_recv.1).set(resource, count-1);
            connection.insert(tick, resource);
        }
    }
}

fn do_connection_send(
    tick: u32,
    connection: &mut Mut<ConnectionLong>,
    ports_send: &ConnectionPortSend,
    ports: &mut Query<&mut Ports>
) {
    if !connection.can_consume(tick) {  return; }
    if let Ok(mut ports) = ports.get_mut(ports_send.0) {
        let resource_head = connection.peek().1;
        let (resource, count) = ports.get(ports_send.1).get_or(resource_head);
        if resource != resource_head { return; }
        ports.get_mut(ports_send.1).set(resource, count+1);
        connection.consume();
    }
}

// TODO move front inside struct?
// TODO port on connection?
// TODO port impls