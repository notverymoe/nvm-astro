/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use bevy::{
    prelude::{Query, Transform, EventReader, Component, Res, ResMut},
    input::{mouse::MouseMotion, keyboard::KeyCode, Input}, core::Time, math::{Vec2, Vec3, Quat, EulerRot}, window::Windows,
};

#[derive(Component)]
pub struct Freecam {
    pub look_speed: f32,
    pub move_speed: f32,
}

pub struct MouseLock(pub bool);

pub fn freecam_system(
    mut q: Query<(&Freecam, &mut Transform)>,
    mut mouse: EventReader<MouseMotion>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mouse_lock: Res<MouseLock>,
    mut windows: ResMut<Windows>,
) {
    let MouseLock(mouse_lock) = *mouse_lock;
    let window = windows.get_primary_mut().unwrap();
    if !mouse_lock || !window.is_focused() {
        window.set_cursor_visibility(true);
        window.set_cursor_lock_mode(false);
        return;
    }

    window.set_cursor_visibility(false);
    window.set_cursor_lock_mode(true);

    let mut look_delta = Vec2::ZERO;
    for event in mouse.iter() { look_delta += event.delta; }
    look_delta *= time.delta_seconds();
 
    let mut move_delta = Vec3::ZERO;
    if keyboard.pressed(KeyCode::W) { move_delta -= Vec3::Z; }
    if keyboard.pressed(KeyCode::A) { move_delta -= Vec3::X; }
    if keyboard.pressed(KeyCode::S) { move_delta += Vec3::Z; }
    if keyboard.pressed(KeyCode::D) { move_delta += Vec3::X; }
    if keyboard.pressed(KeyCode::Q) { move_delta -= Vec3::Y; }
    if keyboard.pressed(KeyCode::E) { move_delta += Vec3::Y; }
    move_delta = move_delta.normalize_or_zero() * time.delta_seconds();
    if keyboard.pressed(KeyCode::LShift  ) { move_delta *= 4.00;  }
    if keyboard.pressed(KeyCode::LControl) { move_delta *= 0.25; }

    if look_delta.length_squared() <= 0.0 && move_delta.length_squared() <= 0.0 {
        return;
    }

    for (camera, mut transform) in q.iter_mut() {
        let look_delta = look_delta * camera.look_speed;
        let (mut y, mut x, _) = transform.rotation.to_euler(EulerRot::YXZ);
        x = (x - look_delta.y).min(core::f32::consts::PI/16.0*7.0).max(-core::f32::consts::PI/16.0*7.0); 
        y -= look_delta.x; 
        transform.rotation = Quat::from_euler(EulerRot::YXZ, y, x, 0.0);

        let move_delta = Quat::from_rotation_y(y) * move_delta * camera.move_speed;
        transform.translation += move_delta;
    }


}