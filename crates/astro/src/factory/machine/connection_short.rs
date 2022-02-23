/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{ResMut, Res, Query, Without, Component, Mut};

use crate::factory::FactoryTick;

use super::{ResourceID, ResourceIDInnerType, ConnectionPortRecv, ConnectionPortSend, Ports};

// TODO this tick lags behind by 16 ticks
//     this allows us to encode only the 
//     timestamp for each packet with 4-bits
//     since the time between two 1s is the
//     max capacity of the connection type
pub struct ConnectionTick(pub u32);

#[derive(Component)]
pub struct ConnectionShort {
    /* -1 */ indicies:   u8,
    /* -8 */ queue:      u64,
    /* 32 */ resources: [ResourceIDInnerType; 16],
    /*----*/
    /* 41 */ // This is bad for that cache line thing right? 64/41 => ~1.3? Ew.
}

impl ConnectionShort {

    pub const fn new(capacity: u8) -> Self {
        assert!(capacity <= 16);
        Self {
            indicies: capacity << 4,
            queue: 0,
            resources: [0; 16],
        }
    }

    pub fn enqueue(&mut self, tick: u32, resource: ResourceID) -> bool {
        match self.is_full() {
            true => false,
            false => {
                unsafe{ self.enqueue_unsafe(tick, resource); }
                true
            }
        }
    }

    pub unsafe fn enqueue_unsafe(&mut self, tick: u32, resource: ResourceID) {
        self.resources[self.len() as usize] = resource.into_inner();
        self.queue |= (tick as u64 & 0x0F) << (self.len() as u64 * 4);
        self.indicies += 1;
    }

    pub fn consume(&mut self) {
        if self.is_empty() { return; }
        unsafe { self.consume_unchecked(); }
    }

    pub unsafe fn consume_unchecked(&mut self) {
        self.indicies -= 1;
        self.queue >>= 4;
        core::ptr::copy(self.resources[1..].as_ptr(), self.resources.as_mut_ptr(), 15);
    }

    pub fn get(&self) -> Option<ResourceID> {
        if self.is_empty() { return None; }
        Some(unsafe{ self.get_unchecked() })
    }

    pub unsafe fn get_unchecked(&self) -> ResourceID {
        ResourceID::from_inner_unchecked(*self.resources.get_unchecked(0))
    }

    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> u8 {
        self.indicies >> 4
    }

    pub fn len(&self) -> u8 {
        self.indicies & 0x0F
    }

    pub fn is_ready_to_consume(&self, tick_factory: u32, tick_connection: u32) -> bool {
        !self.is_empty() && tick_factory >= (tick_connection + (self.queue as u32 & 0x0F))
    }

}

pub fn update_connection_tick(
    mut connection_tick: ResMut<ConnectionTick>,
    factory_tick:        Res<FactoryTick>
) {
    let factory_tick = factory_tick.0 & 0xFFFF_FFF7;
    if connection_tick.0 != factory_tick {
        connection_tick.0 = factory_tick;
    }
}

pub fn connection_short_send_recv(
    factory_tick: Res<FactoryTick>,
    connection_tick: Res<FactoryTick>,
    mut connections: Query<(&mut ConnectionShort, &ConnectionPortRecv, &ConnectionPortSend)>,
    mut ports: Query<&mut Ports>
) {
    let factory_tick = factory_tick.0;
    let connection_tick = connection_tick.0;
    for (mut connection, ports_recv, ports_send) in connections.iter_mut() {
        do_connection_short_send(factory_tick, connection_tick, &mut connection, ports_send, &mut ports);
        do_connection_short_recv(factory_tick, &mut connection, ports_recv, &mut ports);
    }
}

pub fn connection_short_recv(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut ConnectionShort, &ConnectionPortRecv), Without<ConnectionPortSend>>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_recv) in connections.iter_mut() {
        do_connection_short_recv(tick, &mut connection, ports_recv, &mut ports)
    }
}

pub fn connection_short_send(
    factory_tick: Res<FactoryTick>,
    connection_tick: Res<ConnectionTick>,
    mut connections: Query<(&mut ConnectionShort, &ConnectionPortSend), Without<ConnectionPortRecv>>,
    mut ports: Query<&mut Ports>
) {
    let factory_tick = factory_tick.0;
    let connection_tick = connection_tick.0;
    for (mut connection, ports_send) in connections.iter_mut() {
        do_connection_short_send(factory_tick, connection_tick, &mut connection, ports_send, &mut ports)
    }
}

fn do_connection_short_recv(
    factory_tick: u32,
    connection: &mut Mut<ConnectionShort>,
    ports_recv: &ConnectionPortRecv,
    ports: &mut Query<&mut Ports>
) {
    if !connection.is_full() { return; }
    if let Ok(mut ports) = ports.get_mut(ports_recv.0) {
        if let Some((resource, count)) = ports.get(ports_recv.1).get() {
            ports.get_mut(ports_recv.1).set(resource, count-1);
            connection.enqueue(factory_tick, resource);
        }
    }
}

fn do_connection_short_send(
    factory_tick: u32,
    connection_tick: u32,
    connection: &mut Mut<ConnectionShort>,
    ports_send: &ConnectionPortSend,
    ports: &mut Query<&mut Ports>
) {
    if !connection.is_ready_to_consume(factory_tick, connection_tick) {  return; }
    if let Ok(mut ports) = ports.get_mut(ports_send.0) {
        let resource_head = unsafe{ connection.get_unchecked() };
        let (resource, count) = ports.get(ports_send.1).get_or(resource_head);
        if resource != resource_head { return; }
        ports.get_mut(ports_send.1).set(resource, count+1);
        connection.consume();
    }
}