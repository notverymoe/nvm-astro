/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::prelude::{Res, Query};

mod ports;
pub use ports::*;

mod connection;
pub use connection::*;

mod resource;
pub use resource::*;

mod resource_store;
pub use resource_store::*;

use super::FactoryPool;

pub fn connection_updater(
    pool: Res<FactoryPool>, 
    mut q_connections: Query<(&mut Connection,)>, 
    q_ports: Query<&Ports>,
) {
    //for (mut connection,) in q_connections.iter_mut() {
    q_connections.par_for_each_mut(&pool, 1024, |(mut connection,)|{
        connection.tick();
        connection.try_recv(&q_ports);
        connection.try_send(&q_ports);
    });
}