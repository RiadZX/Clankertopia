use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::game_state::GameState;
use crate::world::{ROOM_HALF_X, ROOM_HALF_Z, ROOM_HEIGHT};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Startup, grab_cursor_on_start)
            .add_systems(
                Update,
                (
                    update_cursor_grab,
                    sync_player_orientation,
                    mouse_look.run_if(in_exploring),
                    keyboard_move.run_if(in_exploring),
                ),
            );
    }
}

#[derive(Component, Default)]
pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
}

const EYE_HEIGHT: f32 = 1.65;
const MOVE_SPEED: f32 = 3.2;
const MOUSE_SENS: f32 = 0.0022;
const PLAYER_RADIUS: f32 = 0.3;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, EYE_HEIGHT, 1.0).looking_at(
            Vec3::new(0.0, EYE_HEIGHT, -1.0),
            Vec3::Y,
        ),
    ));
}

fn in_exploring(state: Res<GameState>) -> bool {
    !state.is_typing()
}

fn grab_cursor_on_start(mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if let Ok(mut cursor) = cursors.single_mut() {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }
}

fn sync_player_orientation(
    state: Res<GameState>,
    mut players: Query<(&mut Player, &Transform)>,
) {
    if !state.is_changed() || state.is_typing() {
        return;
    }
    let Ok((mut player, tf)) = players.single_mut() else {
        return;
    };
    let (yaw, pitch, _) = tf.rotation.to_euler(EulerRot::YXZ);
    player.yaw = yaw;
    player.pitch = pitch;
}

fn update_cursor_grab(
    state: Res<GameState>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !state.is_changed() {
        return;
    }
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    if state.is_typing() {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    } else {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }
}

fn mouse_look(
    mut motion: MessageReader<MouseMotion>,
    mut players: Query<(&mut Player, &mut Transform)>,
) {
    let mut delta = Vec2::ZERO;
    for ev in motion.read() {
        delta += ev.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }
    let Ok((mut player, mut tf)) = players.single_mut() else {
        return;
    };
    player.yaw -= delta.x * MOUSE_SENS;
    player.pitch -= delta.y * MOUSE_SENS;
    let limit = std::f32::consts::FRAC_PI_2 - 0.01;
    player.pitch = player.pitch.clamp(-limit, limit);
    tf.rotation = Quat::from_axis_angle(Vec3::Y, player.yaw)
        * Quat::from_axis_angle(Vec3::X, player.pitch);
}

fn keyboard_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<(&Player, &mut Transform)>,
) {
    let Ok((player, mut tf)) = players.single_mut() else {
        return;
    };
    let mut wish = Vec3::ZERO;
    let forward = Quat::from_axis_angle(Vec3::Y, player.yaw) * Vec3::NEG_Z;
    let right = Quat::from_axis_angle(Vec3::Y, player.yaw) * Vec3::X;
    if keys.pressed(KeyCode::KeyW) {
        wish += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        wish -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        wish += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        wish -= right;
    }
    wish.y = 0.0;
    if wish.length_squared() > 0.0 {
        wish = wish.normalize() * MOVE_SPEED * time.delta_secs();
        let new_pos = tf.translation + wish;
        tf.translation.x = new_pos
            .x
            .clamp(-ROOM_HALF_X + PLAYER_RADIUS, ROOM_HALF_X - PLAYER_RADIUS);
        tf.translation.z = new_pos
            .z
            .clamp(-ROOM_HALF_Z + PLAYER_RADIUS, ROOM_HALF_Z - PLAYER_RADIUS);
        tf.translation.y = EYE_HEIGHT.min(ROOM_HEIGHT - 0.1);
    }
}
