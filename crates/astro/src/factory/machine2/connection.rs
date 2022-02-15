/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Entity, Query, Component, Bundle, Mut};

use super::{
    resource::{ResourceIDInnerType, ResourceID}, 
    ports::{PortID, Ports}, 
    ringbuffer::RingBuffer
};

pub type ConnectionDuration = u16;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionHead(u16);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionTail(u16);

#[derive(Component)]
pub struct ConnectionBodyPosition(RingBuffer<ConnectionDuration >);

#[derive(Component)]
pub struct ConnectionBodyResource(RingBuffer<ResourceIDInnerType>);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortRecv(pub Entity, pub PortID);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortSend(pub Entity, pub PortID);

#[derive(Bundle)]
pub struct ConnectionBundle {
    head: ConnectionHead,
    tail: ConnectionTail,
    body_positions: ConnectionBodyPosition,
    body_resources: ConnectionBodyResource,
}

impl ConnectionBundle {
    pub fn new(length: u16) -> Self {
        Self{
            head: ConnectionHead(0),
            tail: ConnectionTail(length),
            body_positions: ConnectionBodyPosition(RingBuffer::new(length)),
            body_resources: ConnectionBodyResource(RingBuffer::new(length)),
        }
    }
}

impl ConnectionHead {
    pub fn can_advance(&self, queue: &ConnectionBodyPosition) -> bool {
        self.0 < queue.0.len() as u16 && *queue.0.get(self.0) <= 1
    }

    pub fn advance(&mut self) {
        self.0 += 1;
    }
}

impl ConnectionTail {
    pub fn is_blocked(&self) -> bool {
        self.0 == 0
    }

    pub fn consume(&mut self) -> u16 {
        std::mem::replace(&mut self.0, 0)
    }

    pub fn advance(&mut self) {
        self.0 += 1;
    }
}

pub fn connection_update(
    mut connections: Query<(&mut ConnectionHead, &mut ConnectionTail, &mut ConnectionBodyPosition, &mut ConnectionBodyResource, &ConnectionPortRecv, &ConnectionPortSend)>,
    mut ports:       Query<&mut Ports>,
) {
    for (mut head, mut tail, mut body_positions, mut body_resources, port_recv, port_send) in connections.iter_mut() {
        do_connection_recv(&mut tail, &mut body_positions, &mut body_resources, port_recv, &mut ports);
        do_connection_send(&mut head, &mut body_positions, &mut body_resources, port_send, &mut ports);
        do_connection_tick(&mut head, &mut tail, &mut body_positions);
    }
}

pub fn connection_recv(
    mut connections: Query<(&mut ConnectionTail, &mut ConnectionBodyPosition, &mut ConnectionBodyResource, &ConnectionPortRecv)>,
    mut ports:       Query<&mut Ports>,
) {
    for (mut tail, mut body_positions, mut body_resources, port_recv) in connections.iter_mut() {
        do_connection_recv(&mut tail, &mut body_positions, &mut body_resources, port_recv, &mut ports)
    }
}

pub fn connection_tick(
    mut connections: Query<(&mut ConnectionHead, &mut ConnectionTail, &mut ConnectionBodyPosition)>,
) {
    for (mut head, mut tail, mut body_positions) in connections.iter_mut() {
        do_connection_tick(&mut head, &mut tail, &mut body_positions);
    }
}

pub fn connection_send(
    mut connections: Query<(&mut ConnectionHead, &mut ConnectionBodyPosition, &mut ConnectionBodyResource, &ConnectionPortSend)>,
    mut ports:       Query<&mut Ports>,
) {
    for (mut head, mut body_positions, mut body_resources, port_send) in connections.iter_mut() {
        do_connection_send(&mut head, &mut body_positions, &mut body_resources, port_send, &mut ports);
    }
}

pub fn do_connection_recv(
    tail: &mut Mut<ConnectionTail>, 
    body_positions: &mut Mut<ConnectionBodyPosition>, 
    body_resources: &mut Mut<ConnectionBodyResource>, 
    port_recv: &ConnectionPortRecv,
    ports: &mut Query<&mut Ports>
) {
    if tail.is_blocked() { 
        println!("Blocked");
        return; 
    }
    if let Ok(mut ports) = ports.get_mut(port_recv.0) {
        if let Some((resource, count)) = ports.get(port_recv.1).get() {
            body_positions.0.push_back(tail.consume());
            body_resources.0.push_back(resource.into_inner());
            ports.get_mut(port_recv.1).set(resource, count - 1);
        }
    }
}

pub fn do_connection_tick(
    head: &mut Mut<ConnectionHead>, 
    tail: &mut Mut<ConnectionTail>, 
    body_positions: &mut Mut<ConnectionBodyPosition>
) {
    if head.0 >= body_positions.0.len() { return; }
    tail.advance();
    *body_positions.0.get_mut(head.0) -= 1;
    if head.can_advance(body_positions) { head.advance(); }
}

fn do_connection_send(
    head: &mut Mut<ConnectionHead>,
    body_positions: &mut Mut<ConnectionBodyPosition>,
    body_resources: &mut Mut<ConnectionBodyResource>,
    port_send: &ConnectionPortSend,
    ports: &mut Query<&mut Ports>,
) {
    if head.0 == 0 { return; }
    if let Ok(mut ports) = ports.get_mut(port_send.0) {
        let head_resource = unsafe{ ResourceID::from_inner_unchecked(*body_resources.0.front()) };
        let (resource, count) = ports.get(port_send.1).get().unwrap_or((head_resource, 0));
        if resource != head_resource { return; }
        ports.get_mut(port_send.1).set(resource, count + 1);

        head.0 = 0;
        body_positions.0.pop_front();
        body_resources.0.pop_front();
    }
}