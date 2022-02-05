/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::{prelude::{Entity, Component, Query}, utils::HashMap};
use nvm_bevyutil::sync::SyncCell;

use super::PowerNetwork;

#[derive(Component, Clone, Copy)]
pub struct PowerDevice{
    pub network: Option<Entity>,
    pub request: u32,
    pub provide: u32,
}

impl PowerDevice {

    pub fn get_network<'a>(&self, q: &'a Query<&PowerNetwork>) -> Option<&'a PowerNetwork> {
        self.network.and_then(|v| q.get(v).ok())
    }

    pub fn apply_network(&self, networks: &Query<&PowerNetwork>) -> bool {
        if let Some(network) = self.get_network(networks) {
            network.change_supply(0, self.provide);
            network.change_demand(0, self.request);
            true
        } else {
            false
        }
    }

    pub fn revert_network(&self, networks: &Query<&PowerNetwork>) -> bool {
        if let Some(network) = self.get_network(networks) {
            network.change_supply(self.provide, 0);
            network.change_demand(self.request, 0);
            true
        } else {
            false
        }
    }

    pub fn update_network(&self, old: PowerDevice, networks: &Query<&PowerNetwork>) -> bool {
        if let Some(network) = self.get_network(networks) {
            network.change_supply(old.provide, self.provide);
            network.change_demand(old.request, self.request);
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct PowerDeviceRegistry {
    lookup: SyncCell<HashMap<Entity, SyncCell<PowerDevice>>>,
}

impl PowerDeviceRegistry {

    pub fn insert(&mut self, k: Entity, v: PowerDevice) {
        self.lookup.borrow_mut().insert(k, v.into());
    }

    pub fn update(&self, k: Entity, v: PowerDevice) -> PowerDevice {
        let lookup   = self.lookup.borrow();
        let mut cell = lookup.get(&k).unwrap().borrow_mut();
        core::mem::replace(&mut *cell, v)
    }

    pub fn remove(&mut self, k: Entity) -> Option<PowerDevice> {
        self.lookup.borrow_mut().remove(&k).map(|v| v.into_inner())
    }

}