use bevy::prelude::*;

use super::builder::DeskOccluder;
use super::config::{CharacterConfig, CharacterShape};
use super::editor::BuiltByOffice;

/// `desk_floor` is the floor anchor of the desk; `desk_entity` is the terminal
/// entity that owns this workstation (used for hide-on-focus tagging).
pub fn spawn_character_and_plaque(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    desk_floor: Vec3,
    yaw: f32,
    name: Option<&str>,
    character: Option<&CharacterConfig>,
    desk_entity: Entity,
) {
    let yaw_q = Quat::from_axis_angle(Vec3::Y, yaw);
    let occluder = DeskOccluder { desk_entity };

    if let Some(cfg) = character {
        spawn_character(
            commands,
            meshes,
            materials,
            desk_floor,
            yaw_q,
            cfg,
            occluder.clone(),
        );
    }
    if let Some(name) = name {
        // The plaque stays visible when focused; just don't tag it as occluder.
        spawn_plaque(commands, meshes, materials, desk_floor, yaw_q, name);
    }
}

fn spawn_character(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    desk_floor: Vec3,
    yaw_q: Quat,
    cfg: &CharacterConfig,
    occluder: DeskOccluder,
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

    // Body: a torso capsule (vertical, length 0.45 from shoulders down to
    // hips), sat on the chair seat (top at y = 0.55 in world). The capsule's
    // local centre is at half its length above the seat.
    let body_len = 0.45_f32;
    let body_radius = 0.18_f32;
    let body_mesh = match cfg.shape {
        CharacterShape::Capsule => meshes.add(Capsule3d::new(body_radius, body_len)),
        CharacterShape::Cube => meshes.add(Cuboid::new(0.40, body_len + body_radius * 2.0, 0.32)),
        CharacterShape::Sphere => meshes.add(Sphere::new(0.30)),
    };
    let head_radius = 0.16_f32;
    let head_mesh = meshes.add(Sphere::new(head_radius));

    // Seat top is at world y = 0.55 (chair pillar 0.45 + half seat thickness
    // 0.05 + a tiny 0.05 for the sitting body); chair is 1.05 m behind the
    // desk along the desk's forward axis.
    let chair_offset = yaw_q * Vec3::new(0.0, 0.0, 1.05);
    let seat_top_world_y = 0.55_f32;
    let body_centre_y = seat_top_world_y + body_len * 0.5 + body_radius;
    let body_pos =
        desk_floor + chair_offset + Vec3::new(0.0, body_centre_y, 0.0);
    let head_pos = body_pos + Vec3::new(0.0, body_len * 0.5 + body_radius + head_radius - 0.04, 0.0);

    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
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
        occluder.clone(),
        Mesh3d(head_mesh),
        MeshMaterial3d(head_mat),
        Transform::from_translation(head_pos),
    ));

    // Arms reaching toward the desk (two short capsules from shoulders to
    // mouse/keyboard area on the desktop). Just a couple of slim cuboids.
    let arm_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(
            (cfg.color[0] + 0.05).clamp(0.0, 1.0),
            (cfg.color[1] + 0.05).clamp(0.0, 1.0),
            (cfg.color[2] + 0.05).clamp(0.0, 1.0),
        ),
        perceptual_roughness: 0.75,
        ..default()
    });
    for sx in [-1.0_f32, 1.0] {
        let arm_mesh = meshes.add(Cuboid::new(0.08, 0.08, 0.55));
        // Shoulder position.
        let shoulder = body_pos
            + yaw_q * Vec3::new(sx * 0.22, body_len * 0.4, 0.0);
        // Arm reaches forward (toward -Z in desk local frame) and slightly
        // down to the desktop in front of the chair (chair_offset 1.05 m
        // forward of the desk, so the desktop is forward of the shoulder).
        let arm_centre = shoulder
            + yaw_q * Vec3::new(0.0, -0.20, -0.55);
        let pitch = -0.6_f32; // forward-down
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(arm_mesh),
            MeshMaterial3d(arm_mat.clone()),
            Transform {
                translation: arm_centre,
                rotation: yaw_q * Quat::from_axis_angle(Vec3::X, pitch),
                scale: Vec3::ONE,
            },
        ));
    }
}

fn spawn_plaque(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    desk_floor: Vec3,
    yaw_q: Quat,
    name: &str,
) {
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
    // Above the monitor (monitor centre y = 1.20, monitor height ~1.0). Place
    // the plaque at y = 1.95, slightly in front of the wall behind the desk.
    let plaque_pos = desk_floor + Vec3::new(0.0, 1.95, 0.0) + yaw_q * Vec3::new(0.0, 0.0, -0.02);
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
