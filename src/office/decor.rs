use bevy::prelude::*;

use super::editor::BuiltByOffice;
use super::textures::ProcTextures;

pub fn spawn_desk_decor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    tex: &ProcTextures,
    desk_world_pos: Vec3,
    yaw: f32,
) {
    let yaw_q = Quat::from_axis_angle(Vec3::Y, yaw);

    // Chair: a low cuboid seat + thin backrest, behind the desk.
    let seat_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.18, 0.22),
        perceptual_roughness: 0.7,
        ..default()
    });
    let seat_mesh = meshes.add(Cuboid::new(0.5, 0.08, 0.5));
    let seat_offset = yaw_q * Vec3::new(0.0, -0.55, 1.05);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(seat_mesh),
        MeshMaterial3d(seat_mat.clone()),
        Transform {
            translation: desk_world_pos + seat_offset,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));
    let back_mesh = meshes.add(Cuboid::new(0.5, 0.55, 0.05));
    let back_offset = yaw_q * Vec3::new(0.0, -0.3, 1.3);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(back_mesh),
        MeshMaterial3d(seat_mat),
        Transform {
            translation: desk_world_pos + back_offset,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));

    // Desk lamp: a stand + a tiny emissive bulb head.
    let lamp_stand_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.35, 0.4),
        base_color_texture: Some(tex.metal.clone()),
        metallic: 0.6,
        perceptual_roughness: 0.3,
        ..default()
    });
    let stand_mesh = meshes.add(Cylinder::new(0.02, 0.35));
    let stand_offset = yaw_q * Vec3::new(0.55, -0.55, 0.05);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(stand_mesh),
        MeshMaterial3d(lamp_stand_mat.clone()),
        Transform {
            translation: desk_world_pos + stand_offset + Vec3::Y * 0.175,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));
    let bulb_mesh = meshes.add(Sphere::new(0.07));
    let bulb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.7),
        emissive: LinearRgba::new(2.5, 2.0, 1.0, 1.0),
        unlit: false,
        ..default()
    });
    let bulb_offset = stand_offset + Vec3::Y * 0.35;
    commands.spawn((
        BuiltByOffice,
        Mesh3d(bulb_mesh),
        MeshMaterial3d(bulb_mat),
        Transform::from_translation(desk_world_pos + bulb_offset),
    ));
    // Warm point light from the bulb.
    commands.spawn((
        BuiltByOffice,
        PointLight {
            intensity: 18_000.0,
            color: Color::srgb(1.0, 0.85, 0.6),
            radius: 0.08,
            range: 4.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(desk_world_pos + bulb_offset),
    ));

    // Potted plant: brown pot + green sphere foliage.
    let pot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.28, 0.18),
        perceptual_roughness: 0.85,
        ..default()
    });
    let pot_mesh = meshes.add(Cylinder::new(0.08, 0.12));
    let pot_offset = yaw_q * Vec3::new(-0.55, -0.55, 0.0);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(pot_mesh),
        MeshMaterial3d(pot_mat),
        Transform::from_translation(desk_world_pos + pot_offset + Vec3::Y * 0.06),
    ));
    let foliage_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.55, 0.25),
        perceptual_roughness: 0.85,
        ..default()
    });
    let foliage_mesh = meshes.add(Sphere::new(0.13));
    commands.spawn((
        BuiltByOffice,
        Mesh3d(foliage_mesh),
        MeshMaterial3d(foliage_mat),
        Transform::from_translation(desk_world_pos + pot_offset + Vec3::Y * 0.22),
    ));

    // Coffee mug.
    let mug_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.95),
        perceptual_roughness: 0.4,
        ..default()
    });
    let mug_mesh = meshes.add(Cylinder::new(0.045, 0.09));
    let mug_offset = yaw_q * Vec3::new(0.35, -0.55, -0.1);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(mug_mesh),
        MeshMaterial3d(mug_mat),
        Transform::from_translation(desk_world_pos + mug_offset + Vec3::Y * 0.045),
    ));
    let coffee_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.12, 0.05),
        perceptual_roughness: 0.6,
        ..default()
    });
    let coffee_mesh = meshes.add(Cylinder::new(0.04, 0.005));
    commands.spawn((
        BuiltByOffice,
        Mesh3d(coffee_mesh),
        MeshMaterial3d(coffee_mat),
        Transform::from_translation(desk_world_pos + mug_offset + Vec3::Y * 0.088),
    ));

    // Keyboard slab in front of the monitor.
    let kb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.1),
        perceptual_roughness: 0.6,
        ..default()
    });
    let kb_mesh = meshes.add(Cuboid::new(0.5, 0.025, 0.16));
    let kb_offset = yaw_q * Vec3::new(0.0, -0.5, -0.15);
    commands.spawn((
        BuiltByOffice,
        Mesh3d(kb_mesh),
        MeshMaterial3d(kb_mat),
        Transform {
            translation: desk_world_pos + kb_offset,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));
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
