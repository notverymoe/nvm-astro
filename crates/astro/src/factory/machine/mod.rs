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
   mut q_connections: Query<(&ConnectionIO, &mut Connection,)>, 
   q_ports: Query<&Ports>,
) {
   //for (connection_ref, mut connection,) in q_connections.iter_mut() {
   q_connections.par_for_each_mut(&pool, 1_000_000, |(connection_ref, mut connection,)|{
       connection.update();
       unsafe{ connection_ref.try_unchecked_recv(&mut connection, &q_ports); }
       unsafe{ connection_ref.try_unchecked_send(&mut connection, &q_ports); }
   });
}