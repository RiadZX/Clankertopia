use bevy::prelude::*;

use super::pty::PtySession;

#[derive(Component)]
pub struct Terminal;

#[derive(Component, Debug, Clone, Copy)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Component)]
pub struct TerminalScreen {
    pub parser: vt100::Parser,
    pub dirty: bool,
}

#[derive(Component)]
pub struct TerminalTexture(pub Handle<Image>);

#[derive(Component)]
pub struct PtyHandle(pub PtySession);

#[derive(Component, Clone, Copy)]
pub struct MonitorQuad {
    pub position: Vec3,
    pub size: Vec2,
}
