/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

const PERF_PRINT_DEBUG:    bool = true;
const PERF_TEST_SIZE:     usize = if PERF_PRINT_DEBUG { 1 } else { 1_000_000 };
const PERF_TEST_MACHINES: usize = PERF_TEST_SIZE*5;
const PERF_SAMPLES:       usize = if PERF_PRINT_DEBUG { 32 } else { 1000 };

use std::time::Instant;
use bevy::{prelude::*, MinimalPlugins, app::AppExit, ecs::event::Events};

use astro::factory::{FactoryPlugins, FactoryStage, spawn_connection, ResourceID, PortID, Ports, ResourceType, ConnectionDuration, ConnectionU4, ConnectionU16, ConnectionQueue, FactoryTick}; 

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
            .add_system_to_stage(     CoreStage::First,  start_timer               )
            .add_system_to_stage(FactoryStage::Machine,  update_passthrough_machine)
            .add_system_to_stage(FactoryStage::Machine,  update_unlimited_source   )
            .add_system_to_stage(      CoreStage::Last, print_chains               )
            .add_system_to_stage(      CoreStage::Last,  auto_exit                 )
            .add_startup_system(setup_performance_test);
    }
}

#[derive(Component)]
pub struct UnlimitedSource(ResourceID);

#[derive(Component, Default)]
pub struct PassthroughMachine;


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
        let producer = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
        let consumer = commands.spawn().insert_bundle(UnlimitedSourceBundle::new(RESOURCE_SPEED.id())).id();
        let passthrough = commands.spawn().insert_bundle(PassthroughMachineBundle::default()).id();
        let conveyor_1 = add_connection(&mut commands,    producer, passthrough, 16);
        let conveyor_2 = add_connection(&mut commands, passthrough,    consumer, 16);

        commands.spawn().insert(ChainView{
            producer,
            conveyor_1,
            passthrough,
            conveyor_2,
            consumer,
        });
    }
}

fn add_connection(commands: &mut Commands, from: Entity, to: Entity, length: ConnectionDuration) -> Entity {
    let mut entity = commands.spawn();
    spawn_connection(&mut entity, length, Some((to, PortID::A)), Some((from, PortID::B)));
    entity.id()
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


#[derive(Component)]
pub struct ChainView {
    producer:    Entity,
    conveyor_1:  Entity,
    passthrough: Entity,
    conveyor_2:  Entity,
    consumer:    Entity,
}

pub fn print_chains(
    tick: Res<FactoryTick>,
    q: Query<&ChainView>,
    q_conveyor: Query<(Option<&ConnectionU4>, Option<&ConnectionU16>)>,
    q_passthrough: Query<&Ports, With<PassthroughMachine>>,
    q_generator: Query<&Ports, With<UnlimitedSource>>,
) {
    if !PERF_PRINT_DEBUG { return; }

    let tick = tick.0;
    for ChainView{producer, consumer, conveyor_1, conveyor_2, passthrough} in q.iter() {
        let producer = format_port(q_generator.get(*producer).unwrap());
        let consumer = format_port(q_generator.get(*consumer).unwrap());
        let conveyor_1 = format_connection(tick, get_connection(q_conveyor.get(*conveyor_1).unwrap()));
        let conveyor_2 = format_connection(tick, get_connection(q_conveyor.get(*conveyor_2).unwrap()));
        let passthrough = format_port(q_passthrough.get(*passthrough).unwrap());
        println!("{}{}{}{}{}", producer, conveyor_1, passthrough, conveyor_2, consumer);
    }

}

static RESOURCE_SPEED: ResourceType = ResourceType::new("SPEED");


fn get_connection<'a, T: ConnectionQueue, R: ConnectionQueue>(val: (Option<&'a T>, Option<&'a R>)) -> &'a dyn ConnectionQueue {
    val.0.map::<&'a dyn ConnectionQueue, _>(|v| v).unwrap_or_else(|| val.1.map::<&'a dyn ConnectionQueue, _>(|v| v).unwrap())
}

fn format_port(port: &Ports) -> String {
    format!("[{: >5}|{: >5}]", port.get(PortID::A).count(), port.get(PortID::B).count() )
}

fn format_connection(tick: u32, connection: &dyn ConnectionQueue) -> String {
    format!("[{}]", connection.resolve(tick).iter().map(|v| if v.is_none() { "░" } else { "█" }).collect::<Vec<&str>>().join(""))
}