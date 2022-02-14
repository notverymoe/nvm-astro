/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Entity, Query, Component, Bundle};

use super::{
    resource::{ResourceIDInnerType, ResourceID}, 
    ports::{PortID, Ports}, 
    ringbuffer::RingBuffer
};

pub type ConnectionDuration = u16;

#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionHead(u16);

#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
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
            head: Default::default(),
            tail: Default::default(),
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

pub fn connection_recv(
    mut connections: Query<(&mut ConnectionTail, &mut ConnectionBodyPosition, &mut ConnectionBodyResource, &ConnectionPortRecv)>,
    mut ports:       Query<&mut Ports>,
) {
    for (
        mut tail,
        mut body_positions,
        mut body_resources,
        port_recv
    ) in connections.iter_mut() {
        if tail.is_blocked() { continue; }
        if let Ok(ports) = ports.get_mut(port_recv.0) {
            if let Some((resource, _)) = ports.get(port_recv.1).get() {
                body_positions.0.push_back(tail.consume());
                body_resources.0.push_back(resource.into_inner());
            }
        }
    }
}

pub fn connecton_tick(
    mut connections: Query<(&mut ConnectionHead, &mut ConnectionTail, &mut ConnectionBodyPosition)>,
) {
    for (mut head, mut tail, mut body_positions) in connections.iter_mut() {
        if head.0 >= body_positions.0.len() { continue; }
        tail.advance();
        *body_positions.0.get_mut(head.0) -= 1;
        if head.can_advance(&body_positions) { head.advance(); }
    }
}

pub fn connection_send(
    mut connections: Query<(&mut ConnectionHead, &mut ConnectionBodyPosition, &mut ConnectionBodyResource, &ConnectionPortSend)>,
    mut ports:       Query<&mut Ports>,
) {
    for (
        mut head,
        mut body_positions,
        mut body_resources,
        port_send
    ) in connections.iter_mut() {
        if head.0 == 0 { continue; }
        if let Ok(mut ports) = ports.get_mut(port_send.0) {
            let head_resource = unsafe{ ResourceID::from_inner_unchecked(*body_resources.0.front()) };
            let (resource, count) = ports.get(port_send.1).get().unwrap_or((head_resource, 0));
            if resource != head_resource { continue; }
            ports.get_mut(port_send.1).set(resource, count + 1);

            head.0 = 0;
            body_positions.0.pop_front();
            body_resources.0.pop_front();
        }
    }
}