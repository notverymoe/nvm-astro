/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

use astro::{factory::FactoryPlugins, client::{freecam_system, Freecam, MouseLock}};
use bevy::{prelude::{Vec3, App, Commands, PerspectiveCameraBundle, shape, Mesh, Color, Transform, Assets, ResMut, AssetServer, Res, OrthographicProjection}, DefaultPlugins, pbr::{StandardMaterial, PbrBundle, DirectionalLight, DirectionalLightBundle}, math::Quat};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FactoryPlugins)
        .insert_resource(MouseLock(true))
        .add_system(freecam_system)
        .add_startup_system(setup_dev)
        .run();
}

fn setup_dev(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    commands.spawn()
        .insert_bundle(DirectionalLightBundle{
            directional_light: DirectionalLight{
                color: Color::WHITE,
                illuminance: 10000.0,
                shadows_enabled: true,
                shadow_depth_bias:   0.2,
                shadow_normal_bias:  0.0,
                shadow_projection: OrthographicProjection {
                    left: -55.00,
                    right: 55.00,
                    bottom: -55.00,
                    top: 55.0,
                    near: -100.0,
                    far: 100.0,
                    ..Default::default()
                }
            },
            transform: Transform::from_rotation(Quat::from_euler(bevy::math::EulerRot::XYZ, -1.0, -1.0, -1.0)),
            ..Default::default()
        });

    commands.spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(2.5, 2.5, 10.0),
            ..Default::default()
        })
        .insert(Freecam{
            look_speed: 0.1,
            move_speed: 10.0,
        });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 100.0 })),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("grass.png")),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::new(0.0,-49.5,0.0)),
        ..Default::default()
    });

}