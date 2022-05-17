/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::{ecs::system::EntityCommands, prelude::{Entity, Component, Without, Query, Res, Mut}};

use crate::factory::{FactoryStageInternal, FactoryTick};

use super::{ResourceID, PortSend, PortRecv, PortID, Ports};

mod simple;
pub use simple::*;

pub trait Pipe {
    /// Enqueues the given resource with the given tick.
    /// 
    /// # Safety 
    /// Must not be called if is_full is true.
    unsafe fn enqueue_unchecked(&mut self, tick: u32, resource: ResourceID);

    /// Contumes the head resource.
    /// 
    /// # Safety 
    /// Must not be called if is_empty is true.
    unsafe fn consume_unchecked(&mut self);

    /// Peeks the head resource.
    /// 
    /// # Safety 
    /// Must not be called if is_empty is true.
    unsafe fn get_unchecked(&self) -> ResourceID;

    fn is_full(&self) -> bool;
    fn is_empty(&self) -> bool;
    fn is_ready_to_consume(&self, tick_factory: u32) -> bool;

    fn resolve(&self, factory_tick: u32) -> Box<[Option<ResourceID>]>;
}

pub fn spawn_pipe(entity: &mut EntityCommands, length: u32, send: Option<(Entity, PortID)>, recv: Option<(Entity, PortID)>) {
    
    entity.insert(PipeSimple::new(length));

    if let Some((e, p)) = send {
        entity.insert(PortSend(e, p));
    }

    if let Some((e, p)) = recv {
        entity.insert(PortRecv(e, p));
    }
}

pub fn register_connection_stage<T: Pipe + Component +>(app: &mut bevy::prelude::App) {
    app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_send_recv::<T>);
    app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_recv::<T>);
    app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_send::<T>);
}

pub fn connection_send_recv<T: Pipe + Component>(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut T, &PortRecv, &PortSend)>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_recv, ports_send) in connections.iter_mut() {
        do_connection_send(tick, &mut connection, ports_send, &mut ports);
        do_connection_recv(tick, &mut connection, ports_recv, &mut ports);
    }
}

pub fn connection_recv<T: Pipe + Component>(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut T, &PortRecv), Without<PortSend>>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_recv) in connections.iter_mut() {
        do_connection_recv(tick, &mut connection, ports_recv, &mut ports)
    }
}

pub fn connection_send<T: Pipe + Component>(
    tick: Res<FactoryTick>,
    mut connections: Query<(&mut T, &PortSend), Without<PortRecv>>,
    mut ports: Query<&mut Ports>
) {
    let tick = tick.0;
    for (mut connection, ports_send) in connections.iter_mut() {
        do_connection_send(tick, &mut connection, ports_send, &mut ports)
    }
}

fn do_connection_recv<T: Pipe>(
    tick: u32,
    connection: &mut Mut<T>,
    ports_recv: &PortRecv,
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

fn do_connection_send<T: Pipe>(
    tick: u32,
    connection: &mut Mut<T>,
    ports_send: &PortSend,
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
