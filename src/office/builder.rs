use bevy::prelude::*;

use super::character::spawn_character_and_plaque;
use super::config::{DoorConfig, OfficeConfig, RoomConfig, Wall};
use super::decor::{spawn_desk_decor, spawn_room_decor};
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
        let monitor_pos =
            origin + Vec3::from_array(desk.position) + Vec3::new(0.0, 1.2, 0.0);
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
        spawn_character_and_plaque(
            commands,
            meshes.as_mut(),
            materials.as_mut(),
            monitor_pos,
            desk.yaw,
            desk.name.as_deref(),
            desk.character.as_ref(),
        );
        // Optional desk surface (cheap rectangle slab) just below the monitor.
        let yaw_q = Quat::from_axis_angle(Vec3::Y, desk.yaw);
        let desk_mat = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.desk_wood.clone()),
            perceptual_roughness: 0.7,
            ..default()
        });
        let desk_mesh = meshes.add(Cuboid::new(1.6, 0.05, 0.7));
        let surface_pos = origin + Vec3::from_array(desk.position) + Vec3::new(0.0, 0.75, 0.0);
        commands.spawn((
            BuiltByOffice,
            Mesh3d(desk_mesh),
            MeshMaterial3d(desk_mat),
            Transform {
                translation: surface_pos,
                rotation: yaw_q,
                scale: Vec3::ONE,
            },
        ));

        spawn_desk_decor(
            commands,
            meshes.as_mut(),
            materials.as_mut(),
            tex,
            monitor_pos,
            desk.yaw,
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
