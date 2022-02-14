/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::{ops::{Add, SubAssign, AddAssign}, mem::MaybeUninit};

use super::ResourceID;

/// Utility class for managing a stored resource, focused on removing the
/// the performance/logic burden of managing an Option<ResourceID> - nessesitated
/// by the type not having an "empty" state - when you're also counting the stored
/// amount.
#[derive(Clone, Copy, Debug)]
pub struct ResourceStore<T> {
    resource: MaybeUninit<ResourceID>,
    stored:   T,
}

impl<T: Default> Default for ResourceStore<T> {
    fn default() -> Self {
        Self {
            resource: MaybeUninit::uninit(),
            stored: Default::default()
        }
    }
}

impl<T: Default + Copy + Eq + Add<Output = T> + AddAssign<T> + SubAssign<T> + Ord> ResourceStore<T> {

    pub fn new(resource: ResourceID, stored: T) -> Self {
        Self{
            resource: MaybeUninit::new(resource), 
            stored
        }
    }

    pub fn resource(&self) -> Option<ResourceID> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe{ self.resource.assume_init() })
        }
    }

    pub fn stored(&self) -> T {
        self.stored
    }

    pub fn set(&mut self, resource: ResourceID, count: T) {
        self.resource = MaybeUninit::new(resource);
        self.stored   = count;
    }

    pub fn clear(&mut self) {
        self.stored = Default::default();
    }

    pub fn pop(&mut self, count: T) {
        self.stored -= count;
    }

    pub fn try_send(&mut self, resource: ResourceID, count: T) -> Result<T, ResourceID> {
        if self.is_empty_or_has(resource) {
            self.set(resource, self.stored + count);
            Ok(count)
        } else {
            Err(unsafe { self.resource.assume_init() })
        }
    }

    pub fn try_recv(&mut self, count: T) -> Option<(ResourceID, T)> {
        if self.stored != Default::default() {
            Some((unsafe{ self.resource.assume_init() }, count.min(self.stored)))
        } else {
            None
        }
    }

    pub fn try_add(&mut self, count: T) -> bool {
        if self.is_empty() {
            false
        } else {
            self.stored += count;
            true
        }
    }

    pub fn is_empty_or_has(&self, resource: ResourceID) -> bool {
        self.resource().map_or(true, |v| v == resource)
    }

    pub fn is_empty(&self) -> bool {
        self.stored == T::default()
    }
}