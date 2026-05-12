use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game_state::GameState;
use crate::player::Player;
use crate::terminal::MonitorQuad;

pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusCameraSnapshot>()
            .add_systems(Update, (focus_system, lock_to_screen_system).chain());
    }
}

const MAX_FOCUS_DISTANCE: f32 = 12.0;

#[derive(Resource, Default)]
pub struct FocusCameraSnapshot {
    pub saved: Option<(Vec3, Quat)>,
    pub target: Option<(Vec3, Quat)>,
}

fn focus_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    mut snapshot: ResMut<FocusCameraSnapshot>,
    camera: Query<(&Transform, &Projection), With<Player>>,
    monitors: Query<(Entity, &MonitorQuad)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    match *state {
        GameState::Exploring => {
            if !keys.just_pressed(KeyCode::KeyE) {
                return;
            }
            let Ok((cam_tf, projection)) = camera.single() else {
                return;
            };
            let origin = cam_tf.translation;

            let mut best: Option<(Entity, &MonitorQuad, f32)> = None;
            for (entity, quad) in monitors.iter() {
                let d = origin.distance(quad.position);
                if d > MAX_FOCUS_DISTANCE {
                    continue;
                }
                if best.map(|(_, _, bd)| d < bd).unwrap_or(true) {
                    best = Some((entity, quad, d));
                }
            }
            let Some((entity, quad, _)) = best else {
                return;
            };

            let aspect = windows
                .single()
                .ok()
                .map(|w| w.width() / w.height().max(1.0))
                .unwrap_or(16.0 / 9.0);

            let fov_y = match projection {
                Projection::Perspective(p) => p.fov,
                _ => std::f32::consts::FRAC_PI_4,
            };
            // Distance such that the full quad fits inside the frustum, with
            // a small padding so edges are visible (and a tiny bit of wall is
            // ok per the user's request).
            let tan_half = (fov_y * 0.5).tan().max(1e-3);
            let dist_for_height = (quad.size.y * 0.5) / tan_half;
            let dist_for_width =
                (quad.size.x * 0.5) / (tan_half * aspect.max(1e-3));
            let fit_dist = dist_for_height.max(dist_for_width) * 1.08;

            snapshot.saved = Some((cam_tf.translation, cam_tf.rotation));
            let target_pos = quad.position + Vec3::Z * fit_dist;
            let target_rot = Transform::from_translation(target_pos)
                .looking_at(quad.position, Vec3::Y)
                .rotation;
            snapshot.target = Some((target_pos, target_rot));
            *state = GameState::Typing(entity);
            eprintln!("[clankertopia] focused terminal {entity:?}");
        }
        GameState::Typing(_) => {
            if keys.just_pressed(KeyCode::Escape) {
                snapshot.target = snapshot.saved.take();
                *state = GameState::Exploring;
                eprintln!("[clankertopia] unfocused terminal");
            }
        }
    }
}

fn lock_to_screen_system(
    time: Res<Time>,
    state: Res<GameState>,
    mut snapshot: ResMut<FocusCameraSnapshot>,
    mut camera: Query<&mut Transform, With<Player>>,
) {
    let Some((tgt_pos, tgt_rot)) = snapshot.target else {
        return;
    };
    let Ok(mut tf) = camera.single_mut() else {
        return;
    };

    let t = (time.delta_secs() * 12.0).min(1.0);
    tf.translation = tf.translation.lerp(tgt_pos, t);
    tf.rotation = tf.rotation.slerp(tgt_rot, t);

    let close = tf.translation.distance(tgt_pos) < 0.01
        && tf.rotation.angle_between(tgt_rot) < 0.01;
    if close {
        tf.translation = tgt_pos;
        tf.rotation = tgt_rot;
        if matches!(*state, GameState::Exploring) {
            snapshot.target = None;
        }
    }
}
