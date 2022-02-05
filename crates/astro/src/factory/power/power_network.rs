/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::{
    sync::atomic::{AtomicU32, Ordering as AtomicOrdering}, 
    cmp::Ordering as CmpOrdering
};

use bevy::prelude::Component;

#[derive(Component)]
pub struct PowerNetwork {
    pub supply_limit: u32,

    supply: AtomicU32,
    demand: AtomicU32,
}

impl PowerNetwork {

    pub fn change_supply(&self, old: u32, new: u32) {
        change(&self.supply, old, new);
    }

    pub fn change_demand(&self, old: u32, new: u32) {
        change(&self.demand, old, new);
    }

    pub fn has_power(&self) -> bool {
        let supply = self.supply.load(AtomicOrdering::Acquire).min(self.supply_limit);
        let demand = self.demand.load(AtomicOrdering::Acquire);
        demand <= supply
    }

}


#[inline(always)] fn change(v: &AtomicU32, old: u32, new: u32) {
    match new.cmp(&old) {
        CmpOrdering::Less    => { v.fetch_sub(old - new, AtomicOrdering::AcqRel); },
        CmpOrdering::Greater => { v.fetch_add(new - old, AtomicOrdering::AcqRel); },
        CmpOrdering::Equal   => {},
    };
}