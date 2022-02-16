/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Entity, Query, Component, Mut, Without, Or};

use super::{
    resource::{ResourceIDInnerType, ResourceID}, 
    ports::{PortID, Ports}, 
    ringbuffer::RingBuffer
};

pub type ConnectionDuration = u16;

pub type ConnectionWithoutBothPorts = Or<(Without<ConnectionPortSend>, Without<ConnectionPortRecv>)>;

#[derive(Component)]
pub struct Connection {
    head: u16,
    tail: u16,
    body: RingBuffer<(ConnectionDuration, ResourceIDInnerType)>
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortRecv(pub Entity, pub PortID);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortSend(pub Entity, pub PortID);


impl Connection {
    pub fn new(length: u16) -> Self {
        Self{
            head: 0,
            tail: length,
            body: RingBuffer::new(length),
        }
    }
}

pub fn connection_update(
    mut connections: Query<(&mut Connection, &ConnectionPortRecv, &ConnectionPortSend)>,
    mut ports:       Query<&mut Ports>,
) {
    for (mut connection, port_recv, port_send) in connections.iter_mut() {
        do_connection_recv(&mut connection, port_recv, &mut ports);
        do_connection_send(&mut connection, port_send, &mut ports);
        do_connection_tick(&mut connection);
    }
}

pub fn connection_recv(
    mut connections: Query<(&mut Connection, &ConnectionPortRecv), Without<ConnectionPortSend>>,
    mut ports:       Query<&mut Ports>,
) {
    for (mut connection, port_recv) in connections.iter_mut() {
        do_connection_recv(&mut connection, port_recv, &mut ports)
    }
}

pub fn connection_tick(
    mut connections: Query<&mut Connection, ConnectionWithoutBothPorts>,
) {
    for mut connection in connections.iter_mut() {
        do_connection_tick(&mut connection);
    }
}

pub fn connection_send(
    mut connections: Query<(&mut Connection, &ConnectionPortSend), Without<ConnectionPortRecv>>,
    mut ports:       Query<&mut Ports>,
) {
    for (mut connection, port_send) in connections.iter_mut() {
        do_connection_send(&mut connection, port_send, &mut ports);
    }
}

#[inline(always)] pub fn do_connection_recv(
    connection: &mut Mut<Connection>,
    port_recv: &ConnectionPortRecv,
    ports: &mut Query<&mut Ports>
) {
    if connection.tail == 0 { return; }
    if let Ok(mut ports) = ports.get_mut(port_recv.0) {
        if let Some((resource, count)) = ports.get(port_recv.1).get() {
            let tail = core::mem::replace(&mut connection.tail, 0);
            connection.body.push_back((tail, resource.into_inner()));
            ports.get_mut(port_recv.1).set(resource, count - 1);
        }
    }
}

#[inline(always)] pub fn do_connection_tick(
    connection: &mut Mut<Connection>
) {
    if connection.head >= connection.body.len() { return; }
    connection.tail += 1;
    let head = connection.head;
    connection.body.get_mut(head).0 -= 1;
    if connection.body.get(head).0 <= 1 { connection.head += 1; }
}

#[inline(always)] fn do_connection_send(
    connection: &mut Mut<Connection>,
    port_send: &ConnectionPortSend,
    ports: &mut Query<&mut Ports>,
) {
    if connection.head == 0 { return; }
    if let Ok(mut ports) = ports.get_mut(port_send.0) {
        let resouce_send = unsafe{ ResourceID::from_inner_unchecked(connection.body.front().0) };
        let (resource_send, count) = ports.get(port_send.1).get_or(resouce_send);
        if resource_send != resouce_send { return; }
        ports.get_mut(port_send.1).set(resource_send, count + 1);

        connection.head = 0;
        connection.body.pop_front();
    }
}