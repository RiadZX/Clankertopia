use bevy::prelude::*;

use super::config::{DoorConfig, OfficeConfig, RoomConfig, Wall};
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
) -> WalkVolumes {
    let world = commands.spawn((OfficeWorld, Transform::default(), Visibility::Visible)).id();
    let mut walks: Vec<WalkBox> = Vec::new();

    for room in &cfg.rooms {
        build_room(commands, meshes, materials, images, world, room, &mut walks);
    }

    // Add door slabs that bridge adjacent rooms so the player can walk through.
    for room in &cfg.rooms {
        for door in &room.doors {
            if let Some(slab) = door_slab(cfg, room, door) {
                walks.push(slab);
            }
        }
    }

    // Always-on ambient lighting.
    commands.spawn((
        DirectionalLight {
            illuminance: 3500.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(2.0, 8.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
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
) {
    let origin = Vec3::from_array(room.origin);
    let [w, h, d] = room.size;

    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(
            room.theme.wall_color[0],
            room.theme.wall_color[1],
            room.theme.wall_color[2],
        ),
        perceptual_roughness: 0.9,
        ..default()
    });
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(
            room.theme.floor_color[0],
            room.theme.floor_color[1],
            room.theme.floor_color[2],
        ),
        perceptual_roughness: 0.95,
        ..default()
    });
    let ceiling_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(
            room.theme.ceiling_color[0],
            room.theme.ceiling_color[1],
            room.theme.ceiling_color[2],
        ),
        perceptual_roughness: 0.95,
        ..default()
    });

    let room_entity = commands
        .spawn((
            RoomEntity { id: room.id.clone() },
            Transform::from_translation(origin),
            Visibility::Visible,
        ))
        .id();
    commands.entity(parent).add_child(room_entity);

    let floor_mesh = meshes.add(Plane3d::default().mesh().size(w, d));
    let floor = commands
        .spawn((
            Mesh3d(floor_mesh.clone()),
            MeshMaterial3d(floor_mat),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    commands.entity(room_entity).add_child(floor);

    let ceiling = commands
        .spawn((
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
            PointLight {
                intensity: 220_000.0,
                radius: 0.5,
                range: w.max(d) * 1.5,
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
        commands.entity(entity).insert(DeskEntity {
            room_id: room.id.clone(),
            desk_id: desk.id.clone(),
        });
        // Optional desk surface (cheap rectangle slab) just below the monitor.
        let yaw_q = Quat::from_axis_angle(Vec3::Y, desk.yaw);
        let desk_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.22, 0.15),
            perceptual_roughness: 0.85,
            ..default()
        });
        let desk_mesh = meshes.add(Cuboid::new(1.6, 0.05, 0.7));
        let surface_pos = origin + Vec3::from_array(desk.position) + Vec3::new(0.0, 0.75, 0.0);
        commands.spawn((
            Mesh3d(desk_mesh),
            MeshMaterial3d(desk_mat),
            Transform {
                translation: surface_pos,
                rotation: yaw_q,
                scale: Vec3::ONE,
            },
        ));
    }

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

        for (a, b) in segments {
            let seg_len = (b - a).max(0.0);
            if seg_len <= 0.001 {
                continue;
            }
            let centre = (a + b) * 0.5;
            let mesh = meshes.add(Plane3d::default().mesh().size(seg_len, h));
            let (x, z) = match wall {
                Wall::North | Wall::South => (centre, fixed),
                Wall::East | Wall::West => (fixed, centre),
            };
            let tilt = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let face = Quat::from_axis_angle(Vec3::Y, yaw);
            let segment = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform {
                        translation: Vec3::new(x, h * 0.5, z),
                        rotation: face * tilt,
                        scale: Vec3::ONE,
                    },
                ))
                .id();
            commands.entity(parent).add_child(segment);
        }

        // Lintels above doors.
        for (offset, width, dh) in openings {
            let centre = offset;
            let lintel_h = (h - dh).max(0.0);
            if lintel_h <= 0.001 {
                continue;
            }
            let mesh = meshes.add(Plane3d::default().mesh().size(width, lintel_h));
            let (x, z) = match wall {
                Wall::North | Wall::South => (centre, fixed),
                Wall::East | Wall::West => (fixed, centre),
            };
            let tilt = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let face = Quat::from_axis_angle(Vec3::Y, yaw);
            let lintel = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform {
                        translation: Vec3::new(x, dh + lintel_h * 0.5, z),
                        rotation: face * tilt,
                        scale: Vec3::ONE,
                    },
                ))
                .id();
            commands.entity(parent).add_child(lintel);
        }
    }
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
    let thickness = 1.5_f32;
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
