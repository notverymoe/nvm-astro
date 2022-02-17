/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::ResMut;

mod connection;
pub use connection::*;

pub fn update_tick(mut tick: ResMut<FactoryTick>) {
    tick.0 += 1;
}