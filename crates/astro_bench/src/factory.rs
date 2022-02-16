/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::time::Instant;
use bevy::{prelude::*, MinimalPlugins, app::{Events, AppExit}};

use astro::factory::{FactoryPlugins, FactoryStage, ResourceID, PortID, Ports, ResourceType, ConnectionDuration, Connection, ConnectionPortRecv, ConnectionPortSend};

pub fn factory_bench() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(FactoryPlugins)
        .add_plugin(FactoryPerfTest)
        .run();
}

pub struct FactoryPerfTest;

impl Plugin for FactoryPerfTest {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(StartTime(None))
            .add_system_to_stage(     CoreStage::First, start_timer               )
            .add_system_to_stage(FactoryStage::Machine, update_passthrough_machine)
            .add_system_to_stage(FactoryStage::Machine, update_unlimited_source   )
            .add_system_to_stage(      CoreStage::Last, auto_exit                 )
            .add_startup_system(setup_performance_test);
    }
}


#[derive(Component)]
pub struct UnlimitedSource(ResourceID);

#[derive(Component, Default)]
pub struct PassthroughMachine;

const PERF_TEST_SIZE:     usize = 6_000_000;
const PERF_TEST_MACHINES: usize = PERF_TEST_SIZE*5;
const PERF_SAMPLES:       usize = 100;

pub struct StartTime(Option<Instant>);

pub fn start_timer(
    mut start: ResMut<StartTime>,
) {
    if start.0.is_none() { start.0 = Some(std::time::Instant::now()); }
}

pub fn auto_exit(
    mut app_exit_events: ResMut<Events<AppExit>>,
    start: Res<StartTime>,
    mut counter: Local<usize>, 
) {
    *counter += 1;
    if *counter < PERF_SAMPLES { return; }
    let run_time = (std::time::Instant::now() - start.0.unwrap()).as_nanos();
    let average: u128 = run_time/((PERF_SAMPLES * PERF_TEST_MACHINES) as u128);
    println!("{}ns per op", average);
    app_exit_events.send(AppExit);
}

pub fn update_passthrough_machine(
    mut q: Query<(&mut Ports,), With<PassthroughMachine>>
) {
    for (mut port,) in q.iter_mut() {
        if let Some((resource, count_send)) = port.get(PortID::A).get() {
            let (resouce_recv, count_recv) = port.get(PortID::B).get().unwrap_or((resource, 0));
            if resouce_recv == resource && count_recv != u16::MAX {
                port.get_mut(PortID::A).set(resource, count_send-1);
                port.get_mut(PortID::B).set(resource, count_recv+1);
            }
        }
    }
}

pub fn update_unlimited_source(
    mut q: Query<(&UnlimitedSource, &mut Ports,)>
) {
    for (UnlimitedSource(resource), mut port,) in q.iter_mut() {
        port.get_mut(PortID::B).set(*resource, 1);
        port.get_mut(PortID::A).clear();
    }
}

pub fn setup_performance_test(mut commands: Commands) {
    for _ in 0..PERF_TEST_SIZE {
        let src = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
        let dst = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
        let passthrough = commands.spawn().insert_bundle(PassthroughMachineBundle::default()).id();
        add_connection(&mut commands,         src, passthrough, 10);
        add_connection(&mut commands, passthrough,         dst, 10);
    }
}

fn add_connection(commands: &mut Commands, from: Entity, to: Entity, length: ConnectionDuration) {
    commands.spawn()
        .insert(Connection::new(length))
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
