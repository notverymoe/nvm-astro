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
    Tick,
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
        app.schedule.add_stage_after(         CoreStage::Update, FactoryStageInternal::Tick, SystemStage::single_threaded());
        app.schedule.add_stage_after(FactoryStageInternal::Tick,        FactoryStage::Power, SystemStage::single_threaded());
        app.schedule.add_stage_after(       FactoryStage::Power,      FactoryStage::Machine, SystemStage::single_threaded());

        app.insert_resource(FactoryTick(0));
        app.schedule.add_system_to_stage(FactoryStageInternal::Tick, update_tick);
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
        register_connection_stage::<ConnectionU4 >(app);
        register_connection_stage::<ConnectionU16>(app);
    }
}