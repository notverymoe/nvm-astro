
use bevy::prelude::{Plugin, SystemStage};

use super::{FactoryStage, FactoryStageInternal};

mod pipe;
pub use pipe::*;

mod ports;
pub use ports::*;

mod resource;
pub use resource::*;

pub struct FactoryResourcePlugin;

impl Plugin for FactoryResourcePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.schedule.add_stage_after(FactoryStage::Machine, FactoryStageInternal::Machine, SystemStage::single_threaded());
        register_connection_stage::<PipeSimple>(app);
    }
}