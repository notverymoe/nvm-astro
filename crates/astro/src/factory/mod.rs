/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{ResMut, PluginGroup, Plugin, CoreStage, SystemStage, StageLabel, ParallelSystemDescriptorCoercion};

mod power;
pub use power::*;

mod machine;
pub use machine::*;

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

pub struct FactoryTick(pub u32);

pub fn update_tick(mut tick: ResMut<FactoryTick>) {
    tick.0 += 1;
}

pub struct FactoryStagePlugin;

impl Plugin for FactoryStagePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(FactoryTick(0));
        app.schedule.add_system_to_stage(CoreStage::First, update_tick);
        app.schedule.add_stage_after(  CoreStage::Update,   FactoryStage::Power, SystemStage::single_threaded());
        app.schedule.add_stage_after(FactoryStage::Power, FactoryStage::Machine, SystemStage::single_threaded());
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
        app.insert_resource(ConnectionTick(0));

        app.schedule.add_stage_after(FactoryStage::Machine, FactoryStageInternal::Machine, SystemStage::single_threaded());
        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_send_recv);
        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_recv.label("recv"));
        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_send.label("send").after("recv"));

        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_short_send_recv.before("tick"));
        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_short_recv.label("recv"));
        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, connection_short_send.label("send").after("recv"));
        app.schedule.add_system_to_stage(FactoryStageInternal::Machine, update_connection_tick.label("tick").after("send"));
    }
}