/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::{prelude::{Component, Entity, Res, Query, Without, Mut}, ecs::system::EntityCommands};

use crate::factory::{FactoryTick, FactoryStageInternal};

use super::{PortID, ConnectionU16, ConnectionU4, ResourceID, Ports};

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortRecv(pub Entity, pub PortID);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortSend(pub Entity, pub PortID);

pub fn spawn_connection(entity: &mut EntityCommands, length: u16, send: Option<(Entity, PortID)>, recv: Option<(Entity, PortID)>) {
    match length {
        length if length <=  16 => entity.insert( ConnectionU4::new(length as usize)),
        //length if length <= 256 => entity.insert( ConnectionU8::new(length as u8)),
        length                  => entity.insert(ConnectionU16::new(length)),
    };

    if let Some((e, p)) = send {
        entity.insert(ConnectionPortSend(e, p));
    }

    if let Some((e, p)) = recv {
        entity.insert(ConnectionPortRecv(e, p));
    }
}

pub trait ConnectionQueue {
    unsafe fn enqueue_unchecked(&mut self, tick: u32, resource: ResourceID);
    unsafe fn consume_unchecked(&mut self);
    unsafe fn get_unchecked(&self) -> ResourceID;
    fn is_full(&self) -> bool;
    fn is_empty(&self) -> bool;
    fn is_ready_to_consume(&self, tick_factory: u32) -> bool;


    fn resolve(&self, factory_tick: u32) -> Box<[Option<ResourceID>]>;
}

pub fn register_connection_stage<T: ConnectionQueue + Component +>(app: &mut bevy::prelude::App) {
    app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_send_recv::<T>);
    app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_recv::<T>);
    app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_send::<T>);
}

pub fn connection_send_recv<T: ConnectionQueue + Component>(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut T, &ConnectionPortRecv, &ConnectionPortSend)>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_recv, ports_send) in connections.iter_mut() {
        do_connection_send(tick, &mut connection, ports_send, &mut ports);
        do_connection_recv(tick, &mut connection, ports_recv, &mut ports);
    }
}

pub fn connection_recv<T: ConnectionQueue + Component>(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut T, &ConnectionPortRecv), Without<ConnectionPortSend>>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_recv) in connections.iter_mut() {
        do_connection_recv(tick, &mut connection, ports_recv, &mut ports)
    }
}

pub fn connection_send<T: ConnectionQueue + Component>(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut T, &ConnectionPortSend), Without<ConnectionPortRecv>>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_send) in connections.iter_mut() {
        do_connection_send(tick, &mut connection, ports_send, &mut ports)
    }
}

fn do_connection_recv<T: ConnectionQueue>(
    tick: u32,
    connection: &mut Mut<T>,
    ports_recv: &ConnectionPortRecv,
    ports: &mut Query<&mut Ports>
) {
    if connection.is_full() { return; }
    if let Ok(mut ports) = ports.get_mut(ports_recv.0) {
        if let Some((resource, count)) = ports.get(ports_recv.1).get() {
            ports.get_mut(ports_recv.1).set(resource, count-1);
            unsafe{ connection.enqueue_unchecked(tick, resource); }
        }
    }
}

fn do_connection_send<T: ConnectionQueue>(
    tick: u32,
    connection: &mut Mut<T>,
    ports_send: &ConnectionPortSend,
    ports: &mut Query<&mut Ports>
) {
    if !connection.is_ready_to_consume(tick) {  return; }
    if let Ok(mut ports) = ports.get_mut(ports_send.0) {
        let resource_head = unsafe{ connection.get_unchecked() };
        let (resource, count) = ports.get(ports_send.1).get_or(resource_head);
        if resource != resource_head { return; }
        ports.get_mut(ports_send.1).set(resource, count+1);
        unsafe{ connection.consume_unchecked() };
    }
}
