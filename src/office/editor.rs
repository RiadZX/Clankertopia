use bevy::prelude::*;

use super::builder::{DeskEntity, WalkVolumes};
use super::config::{
    CharacterConfig, CharacterShape, DeskConfig, OfficeConfig, RoomConfig, RoomTheme,
};
use super::loader::save;
use super::{OfficeConfigPath, OfficeConfigRes};
use crate::game_state::GameState;
use crate::player::Player;

/// Marker for entities spawned by the builder, so they can be cleared on reload.
#[derive(Component)]
pub struct BuiltByOffice;

#[derive(Resource, Default)]
pub struct PendingSave {
    pub dirty: bool,
    pub timer: f32,
}

#[derive(Message)]
pub struct ReloadOffice;

#[derive(Message)]
pub struct RebuildOffice;

pub fn editor_keymap(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    player: Query<&Transform, With<Player>>,
    desks: Query<(Entity, &Transform, &DeskEntity)>,
    mut cfg: ResMut<OfficeConfigRes>,
    mut pending: ResMut<PendingSave>,
    mut reload: MessageWriter<ReloadOffice>,
    mut rebuild: MessageWriter<RebuildOffice>,
) {
    if state.is_typing() {
        return;
    }
    let Ok(player_tf) = player.single() else {
        return;
    };

    // N: add a desk in front of the player, in the room whose AABB contains
    // them. We approximate "current room" by point-in-AABB; if none matches
    // we just pick the first room.
    if keys.just_pressed(KeyCode::KeyN) {
        if add_desk_near_player(&mut cfg.0, player_tf) {
            pending.mark();
            rebuild.write(RebuildOffice);
        }
    }

    // Delete: remove the closest desk within 3 m of the player.
    if keys.just_pressed(KeyCode::Delete) {
        let player_pos = player_tf.translation;
        let mut best: Option<(Entity, f32, String, String)> = None;
        for (e, tf, marker) in desks.iter() {
            let d = tf.translation.distance(player_pos);
            if d > 3.0 {
                continue;
            }
            if best.as_ref().map(|(_, bd, _, _)| d < *bd).unwrap_or(true) {
                best = Some((e, d, marker.room_id.clone(), marker.desk_id.clone()));
            }
        }
        if let Some((_, _, room_id, desk_id)) = best {
            if remove_desk(&mut cfg.0, &room_id, &desk_id) {
                pending.mark();
                rebuild.write(RebuildOffice);
            }
        }
    }

    // R: spawn an adjacent room east of the current room.
    if keys.just_pressed(KeyCode::KeyR) {
        if add_room_east(&mut cfg.0, player_tf) {
            pending.mark();
            rebuild.write(RebuildOffice);
        }
    }

    // Shift+S: force a save now.
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if shift && keys.just_pressed(KeyCode::KeyS) {
        pending.mark();
        pending.timer = 0.0; // immediate flush
    }

    // F5: reload from disk and rebuild.
    if keys.just_pressed(KeyCode::F5) {
        reload.write(ReloadOffice);
    }
}

impl PendingSave {
    fn mark(&mut self) {
        self.dirty = true;
        self.timer = 0.5;
    }
}

pub fn debounced_save(
    time: Res<Time>,
    cfg: Res<OfficeConfigRes>,
    path: Res<OfficeConfigPath>,
    mut pending: ResMut<PendingSave>,
) {
    if !pending.dirty {
        return;
    }
    pending.timer -= time.delta_secs();
    if pending.timer > 0.0 {
        return;
    }
    if let Err(e) = save(&cfg.0, &path.0) {
        eprintln!("[clankertopia] save failed: {e}");
    } else {
        eprintln!("[clankertopia] saved {}", path.0.display());
    }
    pending.dirty = false;
}

pub fn handle_reload(
    mut reload: MessageReader<ReloadOffice>,
    mut rebuild: MessageWriter<RebuildOffice>,
    path: Res<OfficeConfigPath>,
    mut cfg: ResMut<OfficeConfigRes>,
) {
    if reload.read().count() == 0 {
        return;
    }
    match std::fs::read_to_string(&path.0).and_then(|s| {
        ron::from_str::<OfficeConfig>(&s)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }) {
        Ok(new_cfg) => {
            cfg.0 = new_cfg;
            eprintln!("[clankertopia] reloaded {}", path.0.display());
            rebuild.write(RebuildOffice);
        }
        Err(e) => {
            eprintln!("[clankertopia] reload failed: {e}");
        }
    }
}

pub fn handle_rebuild(
    mut rebuild: MessageReader<RebuildOffice>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut walks: ResMut<WalkVolumes>,
    cfg: Res<OfficeConfigRes>,
    built: Query<Entity, With<BuiltByOffice>>,
) {
    if rebuild.read().count() == 0 {
        return;
    }
    for e in built.iter() {
        commands.entity(e).despawn();
    }
    *walks = super::builder::build_office(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        &cfg.0,
    );
}

fn current_room_id(cfg: &OfficeConfig, player_tf: &Transform) -> String {
    let p = player_tf.translation;
    for room in &cfg.rooms {
        let o = Vec3::from_array(room.origin);
        let [w, _, d] = room.size;
        if p.x >= o.x - w * 0.5
            && p.x <= o.x + w * 0.5
            && p.z >= o.z - d * 0.5
            && p.z <= o.z + d * 0.5
        {
            return room.id.clone();
        }
    }
    cfg.rooms
        .first()
        .map(|r| r.id.clone())
        .unwrap_or_default()
}

fn next_desk_id(room: &RoomConfig) -> String {
    let mut n = room.desks.len() + 1;
    loop {
        let candidate = format!("desk-{n}");
        if !room.desks.iter().any(|d| d.id == candidate) {
            return candidate;
        }
        n += 1;
    }
}

fn add_desk_near_player(cfg: &mut OfficeConfig, player_tf: &Transform) -> bool {
    let room_id = current_room_id(cfg, player_tf);
    let Some(room) = cfg.room(&room_id) else {
        return false;
    };
    let origin = Vec3::from_array(room.origin);
    let [w, _, d] = room.size;

    // Position the desk ~1.8 m in front of the player, clamped to inside the room.
    let forward = player_tf.rotation * Vec3::NEG_Z;
    let mut world_pos = player_tf.translation + Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero() * 1.8;
    world_pos.y = 0.0;
    let local = world_pos - origin;
    let margin = 1.0_f32;
    let local = Vec3::new(
        local.x.clamp(-w * 0.5 + margin, w * 0.5 - margin),
        0.0,
        local.z.clamp(-d * 0.5 + margin, d * 0.5 - margin),
    );

    // Face the player.
    let yaw = (player_tf.rotation * Vec3::NEG_Z).xz();
    let yaw = (-yaw.x).atan2(-yaw.y);

    let id = next_desk_id(room);
    let desk = DeskConfig {
        id: id.clone(),
        name: Some(format!("Clanker {}", room.desks.len() + 1)),
        position: [local.x, 0.0, local.z],
        yaw,
        cols: 100,
        rows: 30,
        monitor_size: [1.8, 1.0],
        startup: None,
        character: Some(CharacterConfig {
            color: [
                0.4 + (room.desks.len() as f32 * 0.13).fract() * 0.6,
                0.5,
                0.7,
            ],
            shape: CharacterShape::Capsule,
        }),
    };
    if let Some(r) = cfg.room_mut(&room_id) {
        r.desks.push(desk);
        eprintln!("[clankertopia] added desk {id} in room {room_id}");
        return true;
    }
    false
}

fn remove_desk(cfg: &mut OfficeConfig, room_id: &str, desk_id: &str) -> bool {
    if let Some(room) = cfg.room_mut(room_id) {
        let before = room.desks.len();
        room.desks.retain(|d| d.id != desk_id);
        let removed = room.desks.len() < before;
        if removed {
            eprintln!("[clankertopia] removed desk {desk_id} from room {room_id}");
        }
        return removed;
    }
    false
}

fn add_room_east(cfg: &mut OfficeConfig, player_tf: &Transform) -> bool {
    let from_id = current_room_id(cfg, player_tf);
    let Some(from) = cfg.room(&from_id).cloned() else {
        return false;
    };
    let new_id = {
        let mut n = cfg.rooms.len() + 1;
        loop {
            let candidate = format!("room-{n}");
            if cfg.room(&candidate).is_none() {
                break candidate;
            }
            n += 1;
        }
    };
    let new_size = [8.0_f32, 3.0, 8.0_f32];
    let from_origin = Vec3::from_array(from.origin);
    let from_w = from.size[0];
    let new_origin = [
        from_origin.x + from_w * 0.5 + new_size[0] * 0.5 + 0.001,
        from_origin.y,
        from_origin.z,
    ];

    // Door on east wall of from, west wall of new room.
    let door_width = 1.6_f32;
    let door_height = 2.3_f32;

    if let Some(from_mut) = cfg.room_mut(&from_id) {
        from_mut.doors.push(super::config::DoorConfig {
            wall: super::config::Wall::East,
            offset: 0.0,
            width: door_width,
            height: door_height,
            to_room: new_id.clone(),
        });
    }

    let mut new_room = RoomConfig {
        id: new_id.clone(),
        name: format!("Room {}", cfg.rooms.len() + 1),
        origin: new_origin,
        size: new_size,
        theme: RoomTheme::default(),
        doors: vec![super::config::DoorConfig {
            wall: super::config::Wall::West,
            offset: 0.0,
            width: door_width,
            height: door_height,
            to_room: from_id.clone(),
        }],
        desks: Vec::new(),
    };
    // Inherit theme from origin room to keep the office cohesive.
    new_room.theme = from.theme.clone();
    cfg.rooms.push(new_room);
    eprintln!("[clankertopia] added room {new_id} east of {from_id}");

    let _ = from_id;
    true
}
