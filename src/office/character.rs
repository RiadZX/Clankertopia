use bevy::prelude::*;

use super::config::{CharacterConfig, CharacterShape};
use super::editor::BuiltByOffice;

pub fn spawn_character_and_plaque(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    desk_world_pos: Vec3,
    yaw: f32,
    name: Option<&str>,
    character: Option<&CharacterConfig>,
) {
    let yaw_q = Quat::from_axis_angle(Vec3::Y, yaw);

    if let Some(cfg) = character {
        spawn_character(commands, meshes, materials, desk_world_pos, yaw_q, cfg);
    }
    if let Some(name) = name {
        spawn_plaque(commands, meshes, materials, desk_world_pos, yaw_q, name);
    }
}

fn spawn_character(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    desk_world_pos: Vec3,
    yaw_q: Quat,
    cfg: &CharacterConfig,
) {
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(cfg.color[0], cfg.color[1], cfg.color[2]),
        perceptual_roughness: 0.75,
        ..default()
    });
    let head_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(
            (cfg.color[0] + 0.2).clamp(0.0, 1.0),
            (cfg.color[1] + 0.2).clamp(0.0, 1.0),
            (cfg.color[2] + 0.2).clamp(0.0, 1.0),
        ),
        perceptual_roughness: 0.55,
        ..default()
    });

    let body_mesh = match cfg.shape {
        CharacterShape::Capsule => meshes.add(Capsule3d::new(0.22, 0.55)),
        CharacterShape::Cube => meshes.add(Cuboid::new(0.45, 0.9, 0.4)),
        CharacterShape::Sphere => meshes.add(Sphere::new(0.32)),
    };
    let head_mesh = meshes.add(Sphere::new(0.18));

    // The clanker sits on the FAR side of the desk (the back of the monitor).
    // Local forward is +Z (away from where the player stands to look at the
    // screen). Push them well behind the monitor and lower than the screen so
    // they don't occlude it when you focus.
    let sit_offset = yaw_q * Vec3::new(0.0, -0.55, 0.95);
    let body_pos = desk_world_pos + sit_offset;
    let head_pos = body_pos + yaw_q * Vec3::new(0.0, 0.55, -0.05);

    commands.spawn((
        BuiltByOffice,
        Mesh3d(body_mesh),
        MeshMaterial3d(body_mat),
        Transform {
            translation: body_pos,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));
    commands.spawn((
        BuiltByOffice,
        Mesh3d(head_mesh),
        MeshMaterial3d(head_mat),
        Transform::from_translation(head_pos),
    ));
}

fn spawn_plaque(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    desk_world_pos: Vec3,
    yaw_q: Quat,
    name: &str,
) {
    // A small emissive slab above the monitor, with 3D text in front of it.
    let plaque_w = 0.9_f32;
    let plaque_h = 0.18_f32;
    let plaque_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.05, 0.07),
        emissive: LinearRgba::new(0.04, 0.10, 0.18, 1.0),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let plaque_mesh = meshes.add(Cuboid::new(plaque_w, plaque_h, 0.02));
    // Above the monitor, slightly forward so the player sees it.
    let plaque_pos = desk_world_pos + yaw_q * Vec3::new(0.0, 0.75, -0.02);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(plaque_mesh),
        MeshMaterial3d(plaque_mat),
        Transform {
            translation: plaque_pos,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));

    let face_offset = yaw_q * Vec3::new(0.0, 0.0, 0.012);
    commands.spawn((
        BuiltByOffice,
        Text2d::new(name.to_string()),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.95, 1.0)),
        Transform {
            translation: plaque_pos + face_offset,
            rotation: yaw_q,
            scale: Vec3::splat(0.0035),
        },
    ));
}
