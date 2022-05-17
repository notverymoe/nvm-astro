/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{ResMut, PluginGroup, Plugin, CoreStage, SystemStage, StageLabel};

mod resources;
pub use resources::*;

#[derive(StageLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FactoryStage {
    Machine,
}

pub struct FactoryPlugins;

impl PluginGroup for FactoryPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group.add(FactoryStagePlugin);
        group.add(FactoryResourcePlugin);
    }
}

#[derive(StageLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FactoryStageInternal {
    Tick,
    Machine,
}

pub struct FactoryTick(pub u32);

pub fn update_tick(mut tick: ResMut<FactoryTick>) {
    tick.0 += 1;
}

pub struct FactoryStagePlugin;

impl Plugin for FactoryStagePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.schedule.add_stage_after(         CoreStage::Update, FactoryStageInternal::Tick, SystemStage::single_threaded());
        app.schedule.add_stage_after(FactoryStageInternal::Tick,      FactoryStage::Machine, SystemStage::single_threaded());

        app.insert_resource(FactoryTick(0));
        app.schedule.add_system_to_stage(FactoryStageInternal::Tick, update_tick);
    }
}
