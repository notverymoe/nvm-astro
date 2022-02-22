/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::{prelude::{Component, Entity}, ecs::system::EntityCommands};

use super::{PortID, ConnectionLong, ConnectionShort};

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortRecv(pub Entity, pub PortID);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionPortSend(pub Entity, pub PortID);

pub fn spawn_connection(entity: &mut EntityCommands, length: u16, send: Option<(Entity, PortID)>, recv: Option<(Entity, PortID)>) {
    if length > 16 { 
        entity.insert(ConnectionLong::new(length));
    } else { 
        entity.insert(ConnectionShort::new(length as u8));
    }

    if let Some((e, p)) = send {
        entity.insert(ConnectionPortSend(e, p));
    }

    if let Some((e, p)) = recv {
        entity.insert(ConnectionPortRecv(e, p));
    }
}