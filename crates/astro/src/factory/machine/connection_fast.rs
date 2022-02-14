use bevy::prelude::{Entity, Query, Component, Changed, Bundle, Added, Or};

use super::{ResourceID, PortID, Ports};

pub type ConnectionDuration= u16;

#[derive(Component)]
pub struct ConnectionPacketCounter(u16);



#[derive(Component)]
pub struct ConnectionRecvHead(u16);

#[derive(Component)]
pub struct ConnectionRecvTail(u16);

#[derive(Component)]
pub struct ConnectionRecvBody(Box<[Option<ResourceID>]>);

#[derive(Component)]
pub struct ConnectionRecvPort(pub Entity, pub PortID);



#[derive(Component)]
pub struct ConnectionSendHead(u16);

#[derive(Component)]
pub struct ConnectionSendTail(u16);

#[derive(Component)]
pub struct ConnectionSendBody(Box<[Option<ResourceID>]>);

#[derive(Component)]
pub struct ConnectionSendPort(pub Entity, pub PortID);

#[derive(Bundle)]
pub struct ConnectionBundle {
    packet_counter: ConnectionPacketCounter,
    
    recv_head: ConnectionRecvHead,
    recv_tail: ConnectionRecvTail,
    recv_body: ConnectionRecvBody,

    send_head: ConnectionSendHead,
    send_tail: ConnectionSendTail,
    send_body: ConnectionSendBody,
}

impl ConnectionBundle {

    pub fn new(length: u16) -> Self {
        Self{
            packet_counter: ConnectionPacketCounter(length-1),

            recv_head: ConnectionRecvHead(0),
            recv_tail: ConnectionRecvTail(0),
            recv_body: ConnectionRecvBody(vec![None; length.into()].into_boxed_slice()),
            
            send_head: ConnectionSendHead(0),
            send_tail: ConnectionSendTail(0),
            send_body: ConnectionSendBody(vec![None; length.into()].into_boxed_slice()),
        }
    }

}

pub fn connection_port_check(
    query: Query<(&ConnectionRecvPort, &ConnectionSendPort), Or<(Added<ConnectionRecvPort>, Changed<ConnectionRecvPort>, Added<ConnectionSendPort>, Changed<ConnectionSendPort>)>> 
) {
    for(recv, send) in query.iter() {
        assert!(recv.0 != send.0 || recv.1 != send.1, "Cannot connect to same port on same entity.");
    }
}


pub fn connection_recv(
    mut connections: Query<(&mut ConnectionRecvTail, &mut ConnectionRecvBody, &ConnectionRecvPort, &mut ConnectionPacketCounter)>,
    mut port_query: Query<&mut Ports>,
) {
    for(mut tail, mut body, recv_port, mut counter) in connections.iter_mut() {
        if counter.0 as usize >= body.0.len() { continue; }

        counter.0 += 1;
        tail.0 = wrap_inc(tail.0, body.0.len() as u16);
        body.0[tail.0 as usize] = {
            let mut port = port_query.get_mut(recv_port.0).unwrap();
            let result = port.get(recv_port.1).resource();
            if result.is_some() { port.get_mut(recv_port.1).pop(); }
            result
        };
    }

}

pub fn connection_send(
    mut connections: Query<(&mut ConnectionSendHead, &ConnectionSendTail, &mut ConnectionSendBody, &ConnectionSendPort, &mut ConnectionPacketCounter)>,
    mut ports: Query<&mut Ports>,
) {

    for (mut head, tail, body, send_port, mut counter) in connections.iter_mut() {
        if head.0 == tail.0 { continue; }

        let send = body.0[head.0 as usize].unwrap();

        let mut port = ports.get_mut(send_port.0).unwrap();
        if !port.get(send_port.1).can_send(send, 1) { continue; }
        port.get_mut(send_port.1).send(send, 1).unwrap();

        let len = body.0.len() as u16;
        head.0 = wrap_inc(head.0, len);
        counter.0 -= 1;
    }

}

pub fn connection_tick(
    mut connections: Query<(
        &mut ConnectionSendTail, &mut ConnectionSendBody, 
        &mut ConnectionRecvHead, &ConnectionRecvBody
    )>,
) {
    for (
        mut send_tail, mut send_body, 
        mut recv_head, recv_body
    ) in connections.iter_mut() {

        let connection_len = recv_body.0.len() as u16;
        
        recv_head.0 = wrap_inc(recv_head.0, connection_len);
        if let Some(recv) = recv_body.0[recv_head.0 as usize] {
            send_body.0[send_tail.0 as usize] = Some(recv);
            send_tail.0 = wrap_inc(send_tail.0, connection_len);
        }
    }
}

#[inline] fn wrap_inc(value: u16, max: u16) -> u16 {
    if value + 1 >= max {
        0
    } else {
        value + 1
    }
}