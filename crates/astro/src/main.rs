/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use astro::factory::FactoryPlugins;
use bevy::{prelude::*, MinimalPlugins};
use dev::FactoryPerfTest;

pub mod dev;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(FactoryPlugins)
        .add_plugin(FactoryPerfTest)
        .run();
}
