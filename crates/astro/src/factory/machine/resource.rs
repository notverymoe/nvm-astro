/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use std::{sync::atomic::{AtomicU16, Ordering}, num::NonZeroU16};

use nvm_bevyutil::{compact_str::{CompactStr128}, newtype_compactstr};
use once_cell::sync::OnceCell;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceID(NonZeroU16);

newtype_compactstr!(pub, ResourceUUID, CompactStr128);

pub struct ResourceType {
    id:   OnceCell<ResourceID>,
    uuid: ResourceUUID,
}

impl ResourceType {
    pub const fn new(name: &'static str) -> Self {
        Self{
            id:   OnceCell::new(),
            uuid: ResourceUUID::new(name),
        }
    }

    pub fn id(&self) -> ResourceID {
        *self.id.get_or_init(|| ResourceID(unsafe{
            NonZeroU16::new_unchecked(RESOURCE_UUID.fetch_add(1, Ordering::AcqRel).checked_add(1).expect("Resource UUIDs exhausted"))
        }))
    }

    pub fn uuid(&self) -> ResourceUUID {
        self.uuid
    }
}

static RESOURCE_UUID: AtomicU16 = AtomicU16::new(0);
