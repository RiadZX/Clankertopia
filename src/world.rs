use bevy::prelude::*;

use crate::terminal::{spawn_terminal, MonitorScreen};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}

pub const ROOM_HALF_X: f32 = 5.0;
pub const ROOM_HALF_Z: f32 = 5.0;
pub const ROOM_HEIGHT: f32 = 3.0;

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let room_w = ROOM_HALF_X * 2.0;
    let room_d = ROOM_HALF_Z * 2.0;

    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.18, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.10, 0.12),
        perceptual_roughness: 0.95,
        ..default()
    });
    let ceiling_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.14, 0.18),
        perceptual_roughness: 0.95,
        ..default()
    });

    let floor_mesh = meshes.add(Plane3d::default().mesh().size(room_w, room_d));
    commands.spawn((
        Mesh3d(floor_mesh.clone()),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh3d(floor_mesh.clone()),
        MeshMaterial3d(ceiling_mat),
        Transform::from_xyz(0.0, ROOM_HEIGHT, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
    ));

    let wall_long = meshes.add(Plane3d::default().mesh().size(room_w, ROOM_HEIGHT));
    let wall_short = meshes.add(Plane3d::default().mesh().size(room_d, ROOM_HEIGHT));

    commands.spawn((
        Mesh3d(wall_long.clone()),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, ROOM_HEIGHT * 0.5, -ROOM_HALF_Z)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ));

    commands.spawn((
        Mesh3d(wall_long),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, ROOM_HEIGHT * 0.5, ROOM_HALF_Z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    commands.spawn((
        Mesh3d(wall_short.clone()),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(-ROOM_HALF_X, ROOM_HEIGHT * 0.5, 0.0)
            .with_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2,
            )),
    ));

    commands.spawn((
        Mesh3d(wall_short),
        MeshMaterial3d(wall_mat),
        Transform::from_xyz(ROOM_HALF_X, ROOM_HEIGHT * 0.5, 0.0)
            .with_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                -std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2,
            )),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 3500.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(2.0, 6.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 200_000.0,
            radius: 0.5,
            range: 15.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, ROOM_HEIGHT - 0.3, 0.0),
    ));

    let monitor_pos = Vec3::new(0.0, 1.55, -ROOM_HALF_Z + 0.05);
    let monitor_size = Vec2::new(3.0, 1.7);

    spawn_terminal(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        MonitorScreen {
            position: monitor_pos,
            size: monitor_size,
            cols: 100,
            rows: 30,
        },
    );
}
