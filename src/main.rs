use bevy::prelude::*;

mod focus;
mod game_state;
mod hud;
mod input_router;
mod office;
mod player;
mod terminal;
mod world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Clankertopia".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
        .init_resource::<game_state::GameState>()
        .add_message::<terminal::TerminalResize>()
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            terminal::TerminalPlugin,
            focus::FocusPlugin,
            input_router::InputRouterPlugin,
            hud::HudPlugin,
        ))
        .run();
}
