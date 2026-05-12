use bevy::prelude::*;

use crate::game_state::GameState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, update_hud);
    }
}

#[derive(Component)]
struct HudText;

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        HudText,
        Text::new("[WASD] move   [mouse] look   [E] focus terminal"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 1.0)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
    ));
}

fn update_hud(state: Res<GameState>, mut q: Query<&mut Text, With<HudText>>) {
    let Ok(mut text) = q.single_mut() else {
        return;
    };
    let new = match *state {
        GameState::Exploring => {
            "[WASD] move  [mouse] look  [E] focus  [N] add desk  [Del] remove  [R] add room  [Shift+S] save  [F5] reload"
        }
        GameState::Typing(_) => {
            "[Esc] leave terminal   [Ctrl +/-] zoom   [Ctrl 0] reset zoom"
        }
    };
    if text.0 != new {
        text.0 = new.to_string();
    }
}
