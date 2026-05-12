use bevy::prelude::*;

use super::builder::DeskOccluder;
use crate::game_state::GameState;

pub fn hide_focused_desk_occluders(
    state: Res<GameState>,
    mut q: Query<(&DeskOccluder, &mut Visibility)>,
) {
    let focused = state.focused_entity();
    for (occluder, mut vis) in q.iter_mut() {
        let should_hide = focused == Some(occluder.desk_entity);
        let want = if should_hide {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *vis != want {
            *vis = want;
        }
    }
}
