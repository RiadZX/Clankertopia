use bevy::prelude::*;

use super::character::spawn_character_and_plaque;
use super::config::{DoorConfig, OfficeConfig, RoomConfig, Wall};
use super::decor::{spawn_desk_accessories, spawn_room_decor};
use super::editor::BuiltByOffice;
use super::textures::ProcTextures;
use crate::terminal::{spawn_terminal, MonitorScreen};

#[derive(Component)]
pub struct OfficeWorld;

#[derive(Component)]
pub struct RoomEntity {
    pub id: String,
}

#[derive(Component)]
pub struct DeskEntity {
    pub room_id: String,
    pub desk_id: String,
}

/// Marks meshes (chair, character, desk surface, decor, plaque) that belong
/// to a particular desk and should be hidden while that desk is focused so
/// they don't occlude the screen.
#[derive(Component, Clone)]
pub struct DeskOccluder {
    pub desk_entity: Entity,
}

/// World-space AABB for one walkable area (a room or a door slab connecting
/// two rooms).
#[derive(Debug, Clone, Copy)]
pub struct WalkBox {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Resource, Default)]
pub struct WalkVolumes(pub Vec<WalkBox>);

pub fn build_office(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    cfg: &OfficeConfig,
    tex: &ProcTextures,
) -> WalkVolumes {
    let world = commands
        .spawn((
            OfficeWorld,
            BuiltByOffice,
            Transform::default(),
            Visibility::Visible,
        ))
        .id();
    let mut walks: Vec<WalkBox> = Vec::new();

    for room in &cfg.rooms {
        build_room(commands, meshes, materials, images, world, room, &mut walks, tex);
    }

    // Add door slabs that bridge adjacent rooms so the player can walk through.
    for room in &cfg.rooms {
        for door in &room.doors {
            if let Some(slab) = door_slab(cfg, room, door) {
                walks.push(slab);
            }
        }
    }

    // Bright daylight from above.
    commands.spawn((
        BuiltByOffice,
        DirectionalLight {
            illuminance: 14_000.0,
            color: Color::srgb(1.0, 0.97, 0.92),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    WalkVolumes(walks)
}

fn build_room(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    parent: Entity,
    room: &RoomConfig,
    walks: &mut Vec<WalkBox>,
    tex: &ProcTextures,
) {
    let origin = Vec3::from_array(room.origin);
    let [w, h, d] = room.size;

    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex.plaster.clone()),
        perceptual_roughness: 0.9,
        ..default()
    });
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex.wood_floor.clone()),
        perceptual_roughness: 0.85,
        ..default()
    });
    let ceiling_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex.ceiling_tile.clone()),
        perceptual_roughness: 0.95,
        ..default()
    });

    let room_entity = commands
        .spawn((
            RoomEntity { id: room.id.clone() },
            BuiltByOffice,
            Transform::from_translation(origin),
            Visibility::Visible,
        ))
        .id();
    commands.entity(parent).add_child(room_entity);

    let floor_mesh = meshes.add(Plane3d::default().mesh().size(w, d));
    let floor = commands
        .spawn((
            BuiltByOffice,
            Mesh3d(floor_mesh.clone()),
            MeshMaterial3d(floor_mat),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    commands.entity(room_entity).add_child(floor);

    let ceiling = commands
        .spawn((
            BuiltByOffice,
            Mesh3d(floor_mesh),
            MeshMaterial3d(ceiling_mat),
            Transform::from_xyz(0.0, h, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
        ))
        .id();
    commands.entity(room_entity).add_child(ceiling);

    // Walls with door openings. For simplicity we build each wall as two
    // segments around the door if any door touches that wall.
    build_walls_with_doors(
        commands,
        meshes,
        materials,
        &wall_mat,
        room_entity,
        w,
        h,
        d,
        &room.doors,
    );

    // Ceiling lamp.
    let lamp = commands
        .spawn((
            BuiltByOffice,
            PointLight {
                intensity: 600_000.0,
                color: Color::srgb(1.0, 0.95, 0.85),
                radius: 0.5,
                range: w.max(d) * 2.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, h - 0.3, 0.0),
        ))
        .id();
    commands.entity(room_entity).add_child(lamp);

    // Desks.
    for desk in &room.desks {
        // Floor-relative desk anchor.
        let desk_floor =
            origin + Vec3::from_array(desk.position) + Vec3::new(0.0, 0.0, 0.0);
        // Monitor sits above the desk surface (0.75 m) with a small gap.
        let monitor_pos = desk_floor + Vec3::new(0.0, 1.20, 0.0);
        let entity = spawn_terminal(
            commands,
            meshes,
            materials,
            images,
            MonitorScreen {
                position: monitor_pos,
                yaw: desk.yaw,
                size: Vec2::new(desk.monitor_size[0], desk.monitor_size[1]),
                cols: desk.cols,
                rows: desk.rows,
                startup: desk.startup.clone(),
            },
        );
        commands
            .entity(entity)
            .insert((
                DeskEntity {
                    room_id: room.id.clone(),
                    desk_id: desk.id.clone(),
                },
                BuiltByOffice,
            ));

        spawn_workstation(
            commands,
            meshes,
            materials,
            tex,
            desk_floor,
            desk.yaw,
            entity,
        );

        spawn_character_and_plaque(
            commands,
            meshes.as_mut(),
            materials.as_mut(),
            desk_floor,
            desk.yaw,
            desk.name.as_deref(),
            desk.character.as_ref(),
            entity,
        );
    }

    spawn_room_decor(
        commands,
        meshes.as_mut(),
        materials.as_mut(),
        tex,
        origin,
        room.size,
    );

    // Walkable box for this room (in world space, slightly inset from the
    // walls so the player camera doesn't clip them).
    let inset = 0.3_f32;
    walks.push(WalkBox {
        min: origin + Vec3::new(-w * 0.5 + inset, 0.0, -d * 0.5 + inset),
        max: origin + Vec3::new(w * 0.5 - inset, h, d * 0.5 - inset),
    });
}

fn build_walls_with_doors(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    wall_mat: &Handle<StandardMaterial>,
    parent: Entity,
    w: f32,
    h: f32,
    d: f32,
    doors: &[DoorConfig],
) {
    let walls = [
        (Wall::North, w, -d * 0.5, 0.0, 0.0_f32),
        (Wall::South, w, d * 0.5, 0.0, std::f32::consts::PI),
        (Wall::East, d, w * 0.5, 0.0, std::f32::consts::FRAC_PI_2),
        (Wall::West, d, -w * 0.5, 0.0, -std::f32::consts::FRAC_PI_2),
    ];

    for (wall, length, fixed, _, yaw) in walls {
        let mut openings: Vec<(f32, f32, f32)> = doors
            .iter()
            .filter(|d| d.wall == wall)
            .map(|d| (d.offset, d.width, d.height))
            .collect();
        openings.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut cursor = -length * 0.5;
        let end = length * 0.5;
        let segments = if openings.is_empty() {
            vec![(cursor, end)]
        } else {
            let mut segs = Vec::new();
            for (offset, width, _) in &openings {
                let start = offset - width * 0.5;
                if start > cursor {
                    segs.push((cursor, start));
                }
                cursor = offset + width * 0.5;
            }
            if cursor < end {
                segs.push((cursor, end));
            }
            segs
        };

        let wall_thickness = 0.12_f32;
        let face = Quat::from_axis_angle(Vec3::Y, yaw);

        for (a, b) in segments {
            let seg_len = (b - a).max(0.0);
            if seg_len <= 0.001 {
                continue;
            }
            let centre = (a + b) * 0.5;
            let mesh = meshes.add(Cuboid::new(seg_len, h, wall_thickness));
            let (x, z) = match wall {
                Wall::North | Wall::South => (centre, fixed),
                Wall::East | Wall::West => (fixed, centre),
            };
            let segment = commands
                .spawn((
                    BuiltByOffice,
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform {
                        translation: Vec3::new(x, h * 0.5, z),
                        rotation: face,
                        scale: Vec3::ONE,
                    },
                ))
                .id();
            commands.entity(parent).add_child(segment);
        }

        // Lintels above doors.
        for (offset, width, dh) in &openings {
            let centre = *offset;
            let lintel_h = (h - *dh).max(0.0);
            if lintel_h <= 0.001 {
                continue;
            }
            let mesh = meshes.add(Cuboid::new(*width, lintel_h, wall_thickness));
            let (x, z) = match wall {
                Wall::North | Wall::South => (centre, fixed),
                Wall::East | Wall::West => (fixed, centre),
            };
            let lintel = commands
                .spawn((
                    BuiltByOffice,
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform {
                        translation: Vec3::new(x, dh + lintel_h * 0.5, z),
                        rotation: face,
                        scale: Vec3::ONE,
                    },
                ))
                .id();
            commands.entity(parent).add_child(lintel);
        }

        // Door frames + door leaf for each opening.
        for (offset, width, dh) in &openings {
            spawn_door(
                commands,
                meshes,
                materials,
                parent,
                wall,
                *offset,
                *width,
                *dh,
                fixed,
                face,
            );
        }
    }
}

fn spawn_door(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    wall: Wall,
    offset: f32,
    width: f32,
    height: f32,
    fixed: f32,
    face: Quat,
) {
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.22, 0.15),
        perceptual_roughness: 0.65,
        ..default()
    });
    let door_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.28, 0.18),
        perceptual_roughness: 0.55,
        ..default()
    });
    let handle_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.7, 0.25),
        metallic: 0.85,
        perceptual_roughness: 0.25,
        ..default()
    });

    let frame_t = 0.08_f32;
    let frame_d = 0.18_f32;

    // Side jambs (left and right of door opening).
    for sx in [-1.0_f32, 1.0] {
        let jamb_mesh = meshes.add(Cuboid::new(frame_t, height, frame_d));
        let (x, z) = match wall {
            Wall::North | Wall::South => (offset + sx * (width * 0.5 + frame_t * 0.5), fixed),
            Wall::East | Wall::West => (fixed, offset + sx * (width * 0.5 + frame_t * 0.5)),
        };
        let jamb = commands
            .spawn((
                BuiltByOffice,
                Mesh3d(jamb_mesh),
                MeshMaterial3d(frame_mat.clone()),
                Transform {
                    translation: Vec3::new(x, height * 0.5, z),
                    rotation: face,
                    scale: Vec3::ONE,
                },
            ))
            .id();
        commands.entity(parent).add_child(jamb);
    }
    // Top header of the frame just below the lintel.
    let header_mesh = meshes.add(Cuboid::new(width + frame_t * 2.0, frame_t, frame_d));
    let (hx, hz) = match wall {
        Wall::North | Wall::South => (offset, fixed),
        Wall::East | Wall::West => (fixed, offset),
    };
    let header = commands
        .spawn((
            BuiltByOffice,
            Mesh3d(header_mesh),
            MeshMaterial3d(frame_mat.clone()),
            Transform {
                translation: Vec3::new(hx, height - frame_t * 0.5, hz),
                rotation: face,
                scale: Vec3::ONE,
            },
        ))
        .id();
    commands.entity(parent).add_child(header);

    // Open door leaf, swung 90 degrees into the new room (the door axis is
    // perpendicular to the wall). We hinge it on one jamb.
    let leaf_w = width - 0.02;
    let leaf_h = height - 0.04;
    let leaf_t = 0.04_f32;
    let leaf_mesh = meshes.add(Cuboid::new(leaf_w, leaf_h, leaf_t));
    // Position the leaf centre offset along the door's normal by half leaf_w,
    // so it stands open along the wall's local Z axis.
    let leaf_local_offset = match wall {
        Wall::North => Vec3::new(-width * 0.5, 0.0, -leaf_w * 0.5),
        Wall::South => Vec3::new(width * 0.5, 0.0, leaf_w * 0.5),
        Wall::East => Vec3::new(0.0, 0.0, -width * 0.5 + leaf_w * 0.5),
        Wall::West => Vec3::new(0.0, 0.0, width * 0.5 - leaf_w * 0.5),
    };
    let _ = leaf_local_offset;
    // Simpler: place leaf flat in the doorway plane, just slightly tilted so
    // it looks "ajar".
    let ajar = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4);
    let (lx, lz) = match wall {
        Wall::North | Wall::South => (offset, fixed),
        Wall::East | Wall::West => (fixed, offset),
    };
    let leaf = commands
        .spawn((
            BuiltByOffice,
            Mesh3d(leaf_mesh),
            MeshMaterial3d(door_mat),
            Transform {
                translation: Vec3::new(lx, leaf_h * 0.5, lz),
                rotation: face * ajar,
                scale: Vec3::ONE,
            },
        ))
        .id();
    commands.entity(parent).add_child(leaf);

    // Door handle.
    let handle_mesh = meshes.add(Sphere::new(0.04));
    let handle_local = face * ajar * Vec3::new(leaf_w * 0.4, 0.0, leaf_t * 0.5 + 0.04);
    let handle = commands
        .spawn((
            BuiltByOffice,
            Mesh3d(handle_mesh),
            MeshMaterial3d(handle_mat),
            Transform::from_translation(Vec3::new(lx, leaf_h * 0.5, lz) + handle_local),
        ))
        .id();
    commands.entity(parent).add_child(handle);
}

fn door_slab(cfg: &OfficeConfig, room: &RoomConfig, door: &DoorConfig) -> Option<WalkBox> {
    let other = cfg.room(&door.to_room)?;
    let origin = Vec3::from_array(room.origin);
    let _ = other;
    let [w, _h, d] = room.size;
    let (centre_world, axis_w, axis_d) = match door.wall {
        Wall::North => (
            origin + Vec3::new(door.offset, 0.0, -d * 0.5),
            door.width,
            1.0,
        ),
        Wall::South => (
            origin + Vec3::new(door.offset, 0.0, d * 0.5),
            door.width,
            1.0,
        ),
        Wall::East => (
            origin + Vec3::new(w * 0.5, 0.0, door.offset),
            1.0,
            door.width,
        ),
        Wall::West => (
            origin + Vec3::new(-w * 0.5, 0.0, door.offset),
            1.0,
            door.width,
        ),
    };
    // Slab must be thick enough to bridge the gap between both rooms'
    // inset walk volumes (each room inset 0.3 m + player radius 0.3 m =
    // ~0.6 m clear on each side of the wall). 3.0 m gives plenty of overlap.
    let thickness = 3.0_f32;
    let half_w = axis_w * 0.5;
    let half_d = axis_d * 0.5;
    let (hx, hz) = match door.wall {
        Wall::North | Wall::South => (half_w, thickness * 0.5),
        Wall::East | Wall::West => (thickness * 0.5, half_d),
    };
    Some(WalkBox {
        min: centre_world - Vec3::new(hx, 0.0, hz),
        max: centre_world + Vec3::new(hx, door.height, hz),
    })
}

/// Builds a desk + chair + accessories for one workstation. Everything spawned
/// here is tagged with `DeskOccluder { desk_entity }` so it hides when the
/// player focuses that desk's terminal.
fn spawn_workstation(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tex: &ProcTextures,
    desk_floor: Vec3,
    yaw: f32,
    desk_entity: Entity,
) {
    let occluder = DeskOccluder { desk_entity };
    let yaw_q = Quat::from_axis_angle(Vec3::Y, yaw);

    // ---- Desk ------------------------------------------------------------
    let desk_top_y = 0.75_f32;
    let desk_w = 1.6_f32;
    let desk_d = 0.75_f32;
    let desk_top_t = 0.05_f32;

    let wood_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex.desk_wood.clone()),
        perceptual_roughness: 0.7,
        ..default()
    });
    let dark_wood_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.5, 0.4),
        base_color_texture: Some(tex.desk_wood.clone()),
        perceptual_roughness: 0.75,
        ..default()
    });
    let metal_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.55, 0.6),
        base_color_texture: Some(tex.metal.clone()),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        ..default()
    });

    // Desk top.
    let top_mesh = meshes.add(Cuboid::new(desk_w, desk_top_t, desk_d));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(top_mesh),
        MeshMaterial3d(wood_mat.clone()),
        Transform {
            translation: desk_floor + Vec3::new(0.0, desk_top_y, 0.0),
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));

    // Four legs (metal cuboids).
    let leg_t = 0.05_f32;
    let leg_h = desk_top_y - desk_top_t * 0.5;
    let leg_inset = 0.06_f32;
    let leg_xs = [
        -desk_w * 0.5 + leg_inset + leg_t * 0.5,
        desk_w * 0.5 - leg_inset - leg_t * 0.5,
    ];
    let leg_zs = [
        -desk_d * 0.5 + leg_inset + leg_t * 0.5,
        desk_d * 0.5 - leg_inset - leg_t * 0.5,
    ];
    for lx in leg_xs {
        for lz in leg_zs {
            let leg_mesh = meshes.add(Cuboid::new(leg_t, leg_h, leg_t));
            let local = Vec3::new(lx, leg_h * 0.5, lz);
            commands.spawn((
                BuiltByOffice,
                occluder.clone(),
                Mesh3d(leg_mesh),
                MeshMaterial3d(metal_mat.clone()),
                Transform {
                    translation: desk_floor + yaw_q * local,
                    rotation: yaw_q,
                    scale: Vec3::ONE,
                },
            ));
        }
    }

    // Modesty panel along the back of the desk.
    let panel_mesh = meshes.add(Cuboid::new(desk_w - 0.2, 0.35, 0.02));
    let panel_local = Vec3::new(0.0, desk_top_y - 0.05 - 0.175, desk_d * 0.5 - 0.04);
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(panel_mesh),
        MeshMaterial3d(dark_wood_mat.clone()),
        Transform {
            translation: desk_floor + yaw_q * panel_local,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));

    // ---- Office chair ----------------------------------------------------
    let chair_offset = yaw_q * Vec3::new(0.0, 0.0, 1.05);
    let chair_base = desk_floor + chair_offset;

    let seat_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.18, 0.22),
        perceptual_roughness: 0.7,
        ..default()
    });

    // Caster base: 5 spokes radiating out.
    let pillar_top_y = 0.45_f32;
    let spokes = 5;
    let spoke_len = 0.30_f32;
    let spoke_mesh = meshes.add(Cuboid::new(spoke_len, 0.04, 0.06));
    for i in 0..spokes {
        let a = (i as f32 / spokes as f32) * std::f32::consts::TAU;
        let rot = Quat::from_axis_angle(Vec3::Y, a);
        let local = Vec3::new(spoke_len * 0.5, 0.04, 0.0);
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(spoke_mesh.clone()),
            MeshMaterial3d(metal_mat.clone()),
            Transform {
                translation: chair_base + rot * local,
                rotation: yaw_q * rot,
                scale: Vec3::ONE,
            },
        ));
        // Caster wheel at the end of each spoke.
        let caster_mesh = meshes.add(Sphere::new(0.04));
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(caster_mesh),
            MeshMaterial3d(dark_wood_mat.clone()),
            Transform::from_translation(
                chair_base + rot * Vec3::new(spoke_len, 0.04, 0.0),
            ),
        ));
    }

    // Central pillar.
    let pillar_mesh = meshes.add(Cylinder::new(0.04, pillar_top_y));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(pillar_mesh),
        MeshMaterial3d(metal_mat.clone()),
        Transform::from_translation(chair_base + Vec3::Y * (pillar_top_y * 0.5)),
    ));

    // Seat (top of seat at desk_top_y - 0.30 = 0.45).
    let seat_top_y = pillar_top_y;
    let seat_mesh = meshes.add(Cuboid::new(0.5, 0.10, 0.5));
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(seat_mesh),
        MeshMaterial3d(seat_mat.clone()),
        Transform {
            translation: chair_base + Vec3::Y * (seat_top_y + 0.05),
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));

    // Backrest.
    let back_h = 0.55_f32;
    let back_mesh = meshes.add(Cuboid::new(0.5, back_h, 0.06));
    let back_local = Vec3::new(0.0, seat_top_y + 0.10 + back_h * 0.5, 0.22);
    commands.spawn((
        BuiltByOffice,
        occluder.clone(),
        Mesh3d(back_mesh),
        MeshMaterial3d(seat_mat.clone()),
        Transform {
            translation: chair_base + yaw_q * back_local,
            rotation: yaw_q,
            scale: Vec3::ONE,
        },
    ));

    // Armrests.
    for sx in [-1.0_f32, 1.0] {
        let arm_mesh = meshes.add(Cuboid::new(0.06, 0.05, 0.3));
        let arm_local = Vec3::new(sx * 0.28, seat_top_y + 0.20, 0.05);
        commands.spawn((
            BuiltByOffice,
            occluder.clone(),
            Mesh3d(arm_mesh),
            MeshMaterial3d(seat_mat.clone()),
            Transform {
                translation: chair_base + yaw_q * arm_local,
                rotation: yaw_q,
                scale: Vec3::ONE,
            },
        ));
    }

    // ---- Accessories on the desk ----------------------------------------
    spawn_desk_accessories(
        commands,
        meshes.as_mut(),
        materials.as_mut(),
        tex,
        desk_floor,
        yaw,
        desk_top_y,
        occluder,
    );
}
