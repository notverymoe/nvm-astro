/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::{time::Instant, collections::VecDeque};

use astro::factory::{FactoryStage, FactoryPool, ResourceID, PortID, Ports, ResourceType, ConnectionDuration, ConnectionPortRecv, ConnectionPortSend, ConnectionBundle};
use bevy::prelude::{Query, With, Component, Plugin, Commands, Entity, Bundle, Local, CoreStage, Res};

pub struct FactoryPerfTest;

impl Plugin for FactoryPerfTest {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system_to_stage(FactoryStage::Machine, update_passthrough_machine)
            .add_system_to_stage(FactoryStage::Machine, update_unlimited_source   )
            .add_system_to_stage(      CoreStage::Last, performance_monitor       )
            .add_startup_system(setup_performance_test);
    }
}


#[derive(Component)]
pub struct UnlimitedSource(ResourceID);

#[derive(Component, Default)]
pub struct PassthroughMachine;

const PERF_TEST_SIZE:     usize = 5_000_000;
const PERF_TEST_MACHINES: usize = PERF_TEST_SIZE*5;
const PERF_SAMPLES:       usize = 10;

pub fn performance_monitor(mut inst: Local<VecDeque<u128>>, mut counter: Local<usize>, mut last: Local<Option<Instant>>) {
    if last.is_none() {
        println!("Started.");
        *last = Some(std::time::Instant::now());
    }

    *counter += 1;
    if *counter >= PERF_SAMPLES {
        let now  = std::time::Instant::now();
        inst.push_front((now - last.unwrap()).as_nanos());
        inst.truncate(10);
        *last = Some(now);

        let average: u128 = inst.iter().sum::<u128>()/((*counter * inst.len() * PERF_TEST_MACHINES) as u128);
        println!("{}ns per op", average);
        *counter = 0;
    }
    
    
}

pub fn update_passthrough_machine(
    pool: Res<FactoryPool>, 
    mut q: Query<(&mut Ports,), With<PassthroughMachine>>
) {
    //q.par_for_each_mut(&pool, 1_000_000, |(mut port,)| {
    for (mut port,) in q.iter_mut() {
        if let Some((resource, count_send)) = port.get(PortID::A).get() {
            let (resouce_recv, count_recv) = port.get(PortID::B).get().unwrap_or((resource, 0));
            if resouce_recv == resource && count_recv != u16::MAX {
                port.get_mut(PortID::A).set(resource, count_send-1);
                port.get_mut(PortID::B).set(resource, count_recv+1);
            }
        }
    }//);
}

pub fn update_unlimited_source(
    pool: Res<FactoryPool>, 
    mut q: Query<(&UnlimitedSource, &mut Ports,)>
) {
    //q.par_for_each_mut(&pool, 1_000_000, |(UnlimitedSource(resource), mut port,)| {
    for (UnlimitedSource(resource), mut port,) in q.iter_mut() {
        port.get_mut(PortID::B).set(*resource, 1);
        port.get_mut(PortID::A).clear();
    }//);
}

pub fn setup_performance_test(mut commands: Commands) {
    for _ in 0..PERF_TEST_SIZE {
        let src = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
        let dst = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
        let passthrough = commands.spawn().insert_bundle(PassthroughMachineBundle::default()).id();
        add_connection(&mut commands,         src, passthrough, 10);
        add_connection(&mut commands, passthrough,         dst, 10);
    }
    // for _ in 0..PERF_TEST_SIZE {
    //     let src = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
    //     let dst = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
    //     //let passthrough = commands.spawn().insert_bundle(PassthroughMachineBundle::default()).id();
    //     add_connection(&mut commands, src, dst, 10);
    // }
}

fn add_connection(commands: &mut Commands, from: Entity, to: Entity, length: ConnectionDuration) {
    commands.spawn()
        .insert_bundle(ConnectionBundle::new(length))
        .insert(ConnectionPortRecv(from, PortID::B))
        .insert(ConnectionPortSend(  to, PortID::A));
}

#[derive(Bundle)]
pub struct UnlimitedSourceBundle {
    ports: Ports,
    passthrough: UnlimitedSource,
}


impl UnlimitedSourceBundle {
    pub fn new(resource: ResourceID) -> Self {
        Self{
            ports: Ports::default(),
            passthrough: UnlimitedSource(resource)
        }
    }
}

#[derive(Bundle, Default)]
pub struct PassthroughMachineBundle {
    ports: Ports,
    passthrough: PassthroughMachine,
}

static RESOURCE_SPEED: ResourceType = ResourceType::new("SPEED");