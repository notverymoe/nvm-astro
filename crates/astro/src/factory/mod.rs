/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

//mod machine;
use bevy::{prelude::{PluginGroup, Plugin, CoreStage, SystemStage, StageLabel, ParallelSystemDescriptorCoercion}, tasks::{TaskPool, TaskPoolBuilder}};
//pub use machine::*;

mod power;
pub use power::*;
use shrinkwraprs::Shrinkwrap;

pub mod machine2;
pub use machine2::*;

use self::{connection_send, connection_recv, connection_tick, connection_update};

#[derive(Shrinkwrap)]
pub struct FactoryPool(TaskPool);

#[derive(StageLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FactoryStage {
    Power,
    Machine,
}

pub struct FactoryPlugins;

impl PluginGroup for FactoryPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(FactoryStagePlugin)
            .add(PowerPlugin)
            .add(MachinePlugin);
    }
}


// ///////////////////// //
// // Internal Stages // //
// ///////////////////// //

#[derive(StageLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FactoryStageInternal {
    Power,
    Machine,
}

// //////////////////// //
// // Factory Stages // //
// //////////////////// //

pub struct FactoryStagePlugin;

impl Plugin for FactoryStagePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.schedule.add_stage_after(  CoreStage::Update,   FactoryStage::Power, SystemStage::single_threaded());
        app.schedule.add_stage_after(FactoryStage::Power, FactoryStage::Machine, SystemStage::single_threaded());

        let threads = 4;
        app.insert_resource(FactoryPool(TaskPoolBuilder::new().num_threads(threads).build()));
    }
}

// ////////////////// //
// // Power Plugin // //
// ////////////////// //

pub struct PowerPlugin;

impl Plugin for PowerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.schedule.add_stage_after(FactoryStage::Power, FactoryStageInternal::Power, SystemStage::parallel());
        app.insert_resource(PowerDeviceRegistry::default());

        app.schedule.add_system_to_stage(FactoryStageInternal::Power,   power_update_added.label("prepare").label("added"  ));
        app.schedule.add_system_to_stage(FactoryStageInternal::Power, power_update_removed.label("prepare").label("removed").after("added"  ));

        app.schedule.add_system_to_stage(FactoryStageInternal::Power, power_update_changed.label("update" ).label("changed").after("prepare"));
    }
}

// ///////////////////// //
// // Machine Plugin // //
// //////////////////// //

pub struct MachinePlugin;

impl Plugin for MachinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.schedule.add_stage_after(FactoryStage::Machine, FactoryStageInternal::Machine, SystemStage::single_threaded());
        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_update);
        //app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_tick.label("tick"));
        //app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_recv.label("recv").after("tick"));
        //app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_send.label("send").after("recv"));
    }
}