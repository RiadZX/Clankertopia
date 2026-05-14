use bevy::prelude::*;

use crate::clanker_communication::ClankerCommunicationState;
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

fn update_hud(
    state: Res<GameState>,
    comms: Res<ClankerCommunicationState>,
    mut q: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = q.single_mut() else {
        return;
    };
    let controls = match *state {
        GameState::Exploring => {
            "[WASD] move  [mouse] look  [E] focus  [N] add desk  [Del] remove  [R] add room  [Shift+S] save  [F5] reload"
        }
        GameState::Typing(_) => {
            "[Esc] leave  [Ctrl +/-] zoom  [Ctrl 0] reset  [wheel / Shift+PgUp/PgDn] scroll"
        }
    };
    let mut new = format!(
        "{controls}\n[F7] toggle clanker discussion\n{}",
        comms.banner
    );
    if let Some(verdict) = &comms.latest_verdict {
        let scores = verdict
            .scoreboard
            .iter()
            .take(3)
            .map(|(name, score)| format!("{name}:{score}"))
            .collect::<Vec<_>>()
            .join("  ");
        new.push_str(&format!("\nTop scores: {scores}"));
    }
    if text.0 != new {
        text.0 = new;
    }
}
