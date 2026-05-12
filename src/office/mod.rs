use bevy::prelude::*;
use std::path::PathBuf;

pub mod builder;
pub mod character;
pub mod config;
pub mod decor;
pub mod editor;
pub mod focus_visibility;
pub mod loader;
pub mod textures;

pub use builder::*;
pub use config::*;
pub use editor::{PendingSave, RebuildOffice, ReloadOffice};
pub use loader::*;
pub use textures::ProcTextures;

pub struct OfficePlugin;

#[derive(Resource)]
pub struct OfficeConfigRes(pub OfficeConfig);

#[derive(Resource)]
pub struct OfficeConfigPath(pub PathBuf);

impl Plugin for OfficePlugin {
    fn build(&self, app: &mut App) {
        let (cfg, path) = loader::load_or_init().unwrap_or_else(|e| {
            eprintln!("[clankertopia] failed to load office config: {e}; using built-in default");
            (config::OfficeConfig::default_template(), loader::resolve_write_target())
        });
        app.insert_resource(OfficeConfigRes(cfg))
            .insert_resource(OfficeConfigPath(path))
            .init_resource::<WalkVolumes>()
            .init_resource::<editor::PendingSave>()
            .add_message::<editor::ReloadOffice>()
            .add_message::<editor::RebuildOffice>()
            .add_systems(Startup, spawn_office)
            .add_systems(
                Update,
                (
                    editor::editor_keymap,
                    editor::handle_reload,
                    editor::handle_rebuild,
                    editor::debounced_save,
                )
                    .chain(),
            )
            .add_systems(Update, focus_visibility::hide_focused_desk_occluders);
    }
}

fn spawn_office(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<OfficeConfigRes>,
    mut walks: ResMut<WalkVolumes>,
) {
    let tex = textures::ProcTextures::generate(&mut images);
    *walks = builder::build_office(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        &cfg.0,
        &tex,
    );
    commands.insert_resource(tex);
}
