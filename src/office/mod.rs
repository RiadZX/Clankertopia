use bevy::prelude::*;
use std::path::PathBuf;

pub mod builder;
pub mod config;
pub mod loader;

pub use builder::*;
pub use config::*;
pub use loader::*;

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
            .add_systems(Startup, spawn_office);
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
    *walks = builder::build_office(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        &cfg.0,
    );
}
