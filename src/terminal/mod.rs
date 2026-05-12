use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub mod components;
pub mod pty;
pub mod systems;

pub use components::*;

pub struct TerminalPlugin;

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::drain_pty_system,
                systems::rasterize_system,
                systems::resize_terminal_system,
            )
                .chain(),
        );
    }
}

#[derive(Message)]
pub struct TerminalResize {
    pub entity: Entity,
    pub cols: u16,
    pub rows: u16,
}

pub struct MonitorScreen {
    pub position: Vec3,
    pub yaw: f32,
    pub size: Vec2,
    pub cols: u16,
    pub rows: u16,
    pub startup: Option<crate::office::config::StartupCommand>,
}

/// Embedded Nerd Font (TTF) shipped with the binary; provides Powerline, Devicons,
/// file-type icons, Box Drawing, and full Unicode coverage via cosmic-text shaping.
pub const NERD_FONT_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/NerdFontMono-Regular.ttf");

pub fn spawn_terminal(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    monitor: MonitorScreen,
) -> Entity {
    // Placeholder dimensions; the actual texture is resized to match the
    // cosmic-text rasterizer's pixmap on the first rasterize tick.
    let tex_w = monitor.cols as u32 * 10;
    let tex_h = monitor.rows as u32 * 18;

    let mut image = Image::new_fill(
        Extent3d {
            width: tex_w,
            height: tex_h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[20u8, 24, 32, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    let image_handle = images.add(image);

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(image_handle.clone()),
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let bezel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.02, 0.02, 0.03),
        emissive: LinearRgba::new(0.05, 0.4, 0.6, 1.0),
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let mesh = meshes.add(Rectangle::new(monitor.size.x, monitor.size.y));

    let session = pty::spawn_session(monitor.cols, monitor.rows, monitor.startup.as_ref())
        .expect("failed to spawn PTY session");

    if let Some(start) = &monitor.startup {
        for line in &start.send {
            let _ = session.input_tx.send(line.clone().into_bytes());
        }
    }

    let parser = vt100::Parser::new(monitor.rows, monitor.cols, 10_000);

    let rotation = Quat::from_axis_angle(Vec3::Y, monitor.yaw);
    let transform = Transform {
        translation: monitor.position,
        rotation,
        scale: Vec3::ONE,
    };

    let bezel_mesh = meshes.add(Rectangle::new(
        monitor.size.x + 0.12,
        monitor.size.y + 0.12,
    ));
    let bezel = commands
        .spawn((
            Mesh3d(bezel_mesh),
            MeshMaterial3d(bezel_material),
            Transform::from_xyz(0.0, 0.0, -0.005),
        ))
        .id();

    let entity = commands
        .spawn((
            Terminal,
            TerminalSize {
                cols: monitor.cols,
                rows: monitor.rows,
            },
            TerminalScreen { parser, dirty: true },
            TerminalTexture(image_handle),
            PtyHandle(session),
            MonitorQuad {
                position: monitor.position,
                size: monitor.size,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
        ))
        .id();
    commands.entity(entity).add_child(bezel);
    entity
}
