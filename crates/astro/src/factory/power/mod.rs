/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

mod power_network;
use bevy::prelude::{Query, Entity, RemovedComponents, Res, Changed, Added, ResMut};
pub use power_network::*;

mod power_device;
pub use power_device::*;


// 
// 1. !-- GAME LOGIC --!
//   - Create PowerDevice
//   - Update PowerDevice
//   - Remove PowerDevice
// 2. !-- FACTORY LOGIC --!
//   1. Update PowerNetork
//     - Update requested/supplied
//       - Apply changed PowerDevice
//         - Update registry data
//       - Revert removed PowerDevices via registry
//         - Remove from registry (How? ST? RWLock?)
//   2. Notify machines to update their power requirements
//   3. Apply changed PowerNetwork entries
//   ?. Do we want to generate events for PowerNetworks when they go over limit or under limit?
//   4. Update machines
//     - Machines query PowerNetwork state (ie. has_power)
//       - Generate events?
// 3. !-- GAME LOGIC --!
//   - Create PowerDevice
//   - Update PowerDevice
//   - Remove PowerDevice


pub fn power_update_added(
    mut registry: ResMut<PowerDeviceRegistry>,
    networks:     Query<&PowerNetwork>,
    added:        Query<(Entity, &PowerDevice), Added<PowerDevice>>,
) {
    for (entity, power) in added.iter() {
        registry.insert(entity, *power);
        power.apply_network(&networks);
    }
}

pub fn power_update_removed(
    mut registry: ResMut<PowerDeviceRegistry>,
    networks:     Query<&PowerNetwork>,
    removed:      RemovedComponents<PowerDevice>,
) {
    for entity in removed.iter() {
        if let Some(last) = registry.remove(entity) {
            last.revert_network(&networks);
        } else {
            unreachable!("Attempt to remove already removed component")
        }
    }
}

pub fn power_update_changed(
    registry: Res<PowerDeviceRegistry>,
    networks: Query<&PowerNetwork>,
    changed:  Query<(Entity, &PowerDevice), Changed<PowerDevice>>,
) {
    for (entity, power) in changed.iter() {
        let last = registry.update(entity, *power);
        if last.network != power.network {
            last.revert_network(&networks);
            power.apply_network(&networks);
        } else {
            power.update_network(last, &networks);
        }
    }
}