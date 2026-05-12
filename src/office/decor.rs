use bevy::prelude::*;

use super::builder::DeskOccluder;
use super::editor::BuiltByOffice;
use super::textures::ProcTextures;

/// Spawns the items sitting on top of a desk. All entities are tagged with
/// `BuiltByOffice` for rebuilds and `DeskOccluder` for focus-time hiding.
///
/// `desk_floor` is the desk's anchor at floor level; `desk_top_y` is the
/// height of the desk surface in world units (where accessories sit).
pub fn spawn_desk_accessories(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    tex: &ProcTextures,
    desk_floor: Vec3,
    yaw: f32,
    desk_top_y: f32,
    occluder: DeskOccluder,
) {
    let yaw_q = Quat::from_axis_angle(Vec3::Y, yaw);
    let top_pos = |local: Vec3| desk_floor + yaw_q * Vec3::new(local.x, 0.0, local.z) + Vec3::Y * (desk_top_y + local.y);

    // --- Desk lamp (left-back of desk) ---
    let metal_dark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.20, 0.24),
        base_color_texture: Some(tex.metal.clone()),
        metallic: 0.6,
        perceptual_roughness: 0.35,
        ..default()
    });
    // Base plate.
    let base_mesh = meshes.add(Cylinder::new(0.07, 0.015));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(base_mesh),
        MeshMaterial3d(metal_dark_mat.clone()),
        Transform::from_translation(top_pos(Vec3::new(-0.55, 0.01, 0.18))),
    ));
    // Vertical stand.
    let stand_mesh = meshes.add(Cylinder::new(0.012, 0.36));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(stand_mesh),
        MeshMaterial3d(metal_dark_mat.clone()),
        Transform {
            translation: top_pos(Vec3::new(-0.55, 0.18, 0.18)),
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));
    // Arm.
    let arm_mesh = meshes.add(Cuboid::new(0.012, 0.012, 0.22));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(arm_mesh),
        MeshMaterial3d(metal_dark_mat.clone()),
        Transform {
            translation: top_pos(Vec3::new(-0.55, 0.36, 0.07)),
            rotation: yaw_q * Quat::from_rotation_x(0.4),
            scale: Vec3::ONE,
        },
    ));
    // Shade (cone-ish: small cylinder, emissive yellow underside).
    let shade_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.78, 0.5),
        emissive: LinearRgba::new(1.6, 1.3, 0.7, 1.0),
        ..default()
    });
    let shade_mesh = meshes.add(Cylinder::new(0.07, 0.08));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(shade_mesh),
        MeshMaterial3d(shade_mat),
        Transform {
            translation: top_pos(Vec3::new(-0.55, 0.33, -0.05)),
            rotation: yaw_q * Quat::from_rotation_x(0.6),
            scale: Vec3::ONE,
        },
    ));
    // Warm point light.
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        PointLight {
            intensity: 22_000.0,
            color: Color::srgb(1.0, 0.85, 0.6),
            radius: 0.05,
            range: 3.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(top_pos(Vec3::new(-0.45, 0.30, -0.05))),
    ));

    // --- Potted plant (right-back of desk) ---
    let pot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.50, 0.30, 0.18),
        perceptual_roughness: 0.85,
        ..default()
    });
    let pot_mesh = meshes.add(Cylinder::new(0.08, 0.13));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(pot_mesh),
        MeshMaterial3d(pot_mat),
        Transform::from_translation(top_pos(Vec3::new(0.6, 0.065, 0.20))),
    ));
    let foliage_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.55, 0.25),
        perceptual_roughness: 0.85,
        ..default()
    });
    for (off, r) in [
        (Vec3::new(0.0, 0.20, 0.0), 0.12),
        (Vec3::new(0.05, 0.16, 0.02), 0.08),
        (Vec3::new(-0.04, 0.18, -0.03), 0.08),
    ] {
        let mesh = meshes.add(Sphere::new(r));
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(mesh),
            MeshMaterial3d(foliage_mat.clone()),
            Transform::from_translation(top_pos(Vec3::new(0.6, 0.0, 0.20) + off)),
        ));
    }

    // --- Coffee mug (right of monitor) ---
    let mug_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.92, 0.95),
        perceptual_roughness: 0.4,
        ..default()
    });
    let mug_mesh = meshes.add(Cylinder::new(0.045, 0.10));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(mug_mesh),
        MeshMaterial3d(mug_mat),
        Transform::from_translation(top_pos(Vec3::new(0.45, 0.05, -0.18))),
    ));
    let coffee_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.12, 0.05),
        perceptual_roughness: 0.6,
        ..default()
    });
    let coffee_mesh = meshes.add(Cylinder::new(0.040, 0.005));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(coffee_mesh),
        MeshMaterial3d(coffee_mat),
        Transform::from_translation(top_pos(Vec3::new(0.45, 0.095, -0.18))),
    ));
    // Mug handle.
    let handle_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.92, 0.95),
        perceptual_roughness: 0.4,
        ..default()
    });
    let handle_mesh = meshes.add(Torus::new(0.02, 0.035));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(handle_mesh),
        MeshMaterial3d(handle_mat),
        Transform {
            translation: top_pos(Vec3::new(0.50, 0.05, -0.18)),
            rotation: yaw_q * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            scale: Vec3::ONE,
        },
    ));

    // --- Keyboard ---
    let kb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.10, 0.12),
        perceptual_roughness: 0.6,
        ..default()
    });
    let kb_mesh = meshes.add(Cuboid::new(0.55, 0.02, 0.16));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(kb_mesh),
        MeshMaterial3d(kb_mat.clone()),
        Transform {
            translation: top_pos(Vec3::new(0.0, 0.015, -0.20)),
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));
    // Tiny key bumps (a 12x4 strip of small cuboids).
    for kx in 0..14 {
        for kz in 0..4 {
            let key_mesh = meshes.add(Cuboid::new(0.030, 0.008, 0.030));
            let lx = -0.25 + (kx as f32) * 0.037;
            let lz = -0.20 + 0.055 - (kz as f32) * 0.037;
            commands.spawn((
                BuiltByOffice,
                occluder.clone(),
                Mesh3d(key_mesh),
                MeshMaterial3d(kb_mat.clone()),
                Transform {
                    translation: top_pos(Vec3::new(lx, 0.025, lz)),
                    rotation: yaw_q,
                    scale: Vec3::ONE,
                },
            ));
        }
    }

    // --- Mouse + mouse pad ---
    let pad_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.18, 0.25),
        perceptual_roughness: 0.9,
        ..default()
    });
    let pad_mesh = meshes.add(Cuboid::new(0.22, 0.005, 0.18));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(pad_mesh),
        MeshMaterial3d(pad_mat),
        Transform {
            translation: top_pos(Vec3::new(0.40, 0.0035, -0.20)),
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));
    let mouse_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.12, 0.15),
        perceptual_roughness: 0.55,
        ..default()
    });
    let mouse_mesh = meshes.add(Capsule3d::new(0.035, 0.04));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(mouse_mesh),
        MeshMaterial3d(mouse_mat),
        Transform {
            translation: top_pos(Vec3::new(0.40, 0.03, -0.20)),
            rotation: yaw_q * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            scale: Vec3::ONE,
        },
    ));

    // --- Stack of books on the left edge of the desk ---
    let book_colors = [
        Color::srgb(0.65, 0.18, 0.18),
        Color::srgb(0.20, 0.35, 0.60),
        Color::srgb(0.85, 0.65, 0.20),
    ];
    for (i, c) in book_colors.iter().enumerate() {
        let mat = materials.add(StandardMaterial {
            base_color: *c,
            perceptual_roughness: 0.6,
            ..default()
        });
        let mesh = meshes.add(Cuboid::new(0.16, 0.035, 0.22));
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform {
                translation: top_pos(Vec3::new(
                    -0.55,
                    0.0175 + (i as f32) * 0.038,
                    -0.18,
                )),
                rotation: yaw_q * Quat::from_axis_angle(Vec3::Y, 0.08 * i as f32),
                scale: Vec3::ONE,
            },
        ));
    }

    // --- Picture frame standing on the desk ---
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.30, 0.20),
        perceptual_roughness: 0.7,
        ..default()
    });
    let frame_mesh = meshes.add(Cuboid::new(0.18, 0.13, 0.012));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(frame_mesh),
        MeshMaterial3d(frame_mat),
        Transform {
            translation: top_pos(Vec3::new(-0.30, 0.10, 0.18)),
            rotation: yaw_q * Quat::from_rotation_x(0.18),
            scale: Vec3::ONE,
        },
    ));
    let photo_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex.poster_grid.clone()),
        ..default()
    });
    let photo_mesh = meshes.add(Cuboid::new(0.15, 0.11, 0.001));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(photo_mesh),
        MeshMaterial3d(photo_mat),
        Transform {
            translation: top_pos(Vec3::new(-0.30, 0.10, 0.175)),
            rotation: yaw_q * Quat::from_rotation_x(0.18),
            scale: Vec3::ONE,
        },
    ));

    // --- Sticky notes (3 small colored squares) attached to the monitor base ---
    let note_colors = [
        Color::srgb(1.0, 0.95, 0.55),
        Color::srgb(0.6, 0.95, 0.6),
        Color::srgb(0.95, 0.65, 0.85),
    ];
    for (i, c) in note_colors.iter().enumerate() {
        let mat = materials.add(StandardMaterial {
            base_color: *c,
            unlit: true,
            perceptual_roughness: 0.9,
            ..default()
        });
        let mesh = meshes.add(Cuboid::new(0.06, 0.001, 0.06));
        let lx = -0.06 + (i as f32) * 0.07;
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform {
                translation: top_pos(Vec3::new(lx, 0.001, -0.04)),
                rotation: yaw_q * Quat::from_axis_angle(Vec3::Y, 0.15 * (i as f32 - 1.0)),
                scale: Vec3::ONE,
            },
        ));
    }

    // --- Pen holder + pens ---
    let holder_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.30, 0.30, 0.35),
        base_color_texture: Some(tex.metal.clone()),
        metallic: 0.4,
        perceptual_roughness: 0.45,
        ..default()
    });
    let holder_mesh = meshes.add(Cylinder::new(0.05, 0.10));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(holder_mesh),
        MeshMaterial3d(holder_mat),
        Transform::from_translation(top_pos(Vec3::new(-0.45, 0.05, -0.10))),
    ));
    let pen_colors = [
        Color::srgb(0.10, 0.10, 0.10),
        Color::srgb(0.20, 0.50, 0.85),
        Color::srgb(0.90, 0.20, 0.25),
    ];
    for (i, c) in pen_colors.iter().enumerate() {
        let mat = materials.add(StandardMaterial {
            base_color: *c,
            perceptual_roughness: 0.45,
            ..default()
        });
        let mesh = meshes.add(Cylinder::new(0.005, 0.16));
        let off_x = -0.45 + (i as f32 - 1.0) * 0.018;
        let off_z = -0.10 + (i as f32 - 1.0) * 0.012;
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform {
                translation: top_pos(Vec3::new(off_x, 0.14, off_z)),
                rotation: yaw_q
                    * Quat::from_axis_angle(Vec3::X, 0.12 * (i as f32 - 1.0)),
                scale: Vec3::ONE,
            },
        ));
    }
}

pub fn spawn_room_decor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    tex: &ProcTextures,
    room_origin: Vec3,
    room_size: [f32; 3],
) {
    let [w, h, d] = room_size;

    // Central rug on the floor.
    let rug_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex.rug_pattern.clone()),
        perceptual_roughness: 0.95,
        ..default()
    });
    let rug_size = w.min(d) * 0.5;
    let rug_mesh = meshes.add(Plane3d::default().mesh().size(rug_size, rug_size));
    commands.spawn((
        BuiltByOffice,
        Mesh3d(rug_mesh),
        MeshMaterial3d(rug_mat),
        Transform::from_translation(room_origin + Vec3::Y * 0.005),
    ));

    // Posters on the south wall (player spawn side), evenly spaced.
    let poster_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex.poster_grid.clone()),
        unlit: false,
        perceptual_roughness: 0.6,
        ..default()
    });
    let poster_count = ((w / 3.0).floor() as i32).max(1);
    for i in 0..poster_count {
        let t = (i as f32 + 0.5) / poster_count as f32;
        let x = -w * 0.5 + w * t;
        let poster_mesh = meshes.add(Plane3d::default().mesh().size(0.7, 1.0));
        commands.spawn((
            BuiltByOffice,
            Mesh3d(poster_mesh),
            MeshMaterial3d(poster_mat.clone()),
            Transform {
                translation: room_origin + Vec3::new(x, h * 0.6, d * 0.5 - 0.02),
                rotation: Quat::from_euler(
                    EulerRot::YXZ,
                    std::f32::consts::PI,
                    std::f32::consts::FRAC_PI_2,
                    0.0,
                ),
                scale: Vec3::ONE,
            },
        ));
    }

    // Floor lamp in a corner.
    let lamp_stand_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.32),
        base_color_texture: Some(tex.metal.clone()),
        metallic: 0.5,
        perceptual_roughness: 0.35,
        ..default()
    });
    let stand_mesh = meshes.add(Cylinder::new(0.04, 1.6));
    let corner = room_origin + Vec3::new(-w * 0.5 + 0.6, 0.8, -d * 0.5 + 0.6);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(stand_mesh),
        MeshMaterial3d(lamp_stand_mat),
        Transform::from_translation(corner),
    ));
    let shade_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.92, 0.8),
        emissive: LinearRgba::new(1.5, 1.2, 0.7, 1.0),
        ..default()
    });
    let shade_mesh = meshes.add(Cylinder::new(0.18, 0.25));
    commands.spawn((
        BuiltByOffice,
        Mesh3d(shade_mesh),
        MeshMaterial3d(shade_mat),
        Transform::from_translation(corner + Vec3::Y * 0.85),
    ));
    commands.spawn((
        BuiltByOffice,
        PointLight {
            intensity: 35_000.0,
            color: Color::srgb(1.0, 0.9, 0.75),
            range: 6.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(corner + Vec3::Y * 0.85),
    ));

    // Big plant in the opposite corner.
    let big_pot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.25, 0.18),
        perceptual_roughness: 0.85,
        ..default()
    });
    let big_pot_mesh = meshes.add(Cylinder::new(0.22, 0.35));
    let plant_corner = room_origin + Vec3::new(w * 0.5 - 0.6, 0.175, -d * 0.5 + 0.6);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(big_pot_mesh),
        MeshMaterial3d(big_pot_mat),
        Transform::from_translation(plant_corner),
    ));
    let big_foliage_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.5, 0.22),
        perceptual_roughness: 0.85,
        ..default()
    });
    for (off, r) in [
        (Vec3::new(0.0, 0.55, 0.0), 0.35),
        (Vec3::new(0.18, 0.45, 0.05), 0.22),
        (Vec3::new(-0.15, 0.5, -0.05), 0.22),
        (Vec3::new(0.05, 0.75, -0.08), 0.20),
    ] {
        let mesh = meshes.add(Sphere::new(r));
        commands.spawn((
            BuiltByOffice,
            Mesh3d(mesh),
            MeshMaterial3d(big_foliage_mat.clone()),
            Transform::from_translation(plant_corner + off),
        ));
    }
}
