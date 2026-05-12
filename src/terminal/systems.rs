use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use ratatui::Terminal;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color as RtColor, Modifier as RtModifier, Style as RtStyle};
use ratatui::widgets::Widget;
use soft_ratatui::{CosmicText, SoftBackend};

use super::components::*;
use super::{TerminalResize, NERD_FONT_BYTES};

/// Default font pixel size (cosmic-text "metrics" font size). Effective glyph
/// width/height are derived by cosmic-text from this and the font's metrics.
pub const DEFAULT_FONT_SIZE: i32 = 16;

#[derive(Component)]
pub struct TerminalRenderer {
    pub terminal: Terminal<SoftBackend<CosmicText>>,
    pub frames: u32,
    pub font_size: i32,
}

pub fn drain_pty_system(
    mut commands: Commands,
    mut q: Query<(
        Entity,
        &mut TerminalScreen,
        &PtyHandle,
        Option<&mut TerminalRenderer>,
        &TerminalSize,
    )>,
) {
    for (entity, mut screen, pty, renderer, size) in q.iter_mut() {
        let mut got = false;
        while let Ok(chunk) = pty.0.bytes_rx.try_recv() {
            screen.parser.process(&chunk);
            got = true;
        }
        if got {
            // Snap to bottom on new PTY output so the user doesn't miss
            // anything while scrolled up.
            if screen.parser.screen().scrollback() != 0 {
                screen.parser.screen_mut().set_scrollback(0);
            }
            screen.dirty = true;
        }
        if renderer.is_none() {
            let backend = SoftBackend::<CosmicText>::new(
                size.cols,
                size.rows,
                DEFAULT_FONT_SIZE,
                NERD_FONT_BYTES,
            );
            let mut terminal = Terminal::new(backend).expect("ratatui terminal");
            let _ = terminal.clear();
            commands.entity(entity).insert(TerminalRenderer {
                terminal,
                frames: 0,
                font_size: DEFAULT_FONT_SIZE,
            });
            screen.dirty = true;
        }
    }
}

struct VtScreenWidget<'a> {
    screen: &'a vt100::Screen,
    rows: u16,
    cols: u16,
}

impl<'a> Widget for VtScreenWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.rows.min(area.height);
        let cols = self.cols.min(area.width);
        for row in 0..rows {
            for col in 0..cols {
                let dst_x = area.x + col;
                let dst_y = area.y + row;
                let Some(dst) = buf.cell_mut((dst_x, dst_y)) else {
                    continue;
                };
                let Some(cell) = self.screen.cell(row, col) else {
                    dst.set_symbol(" ");
                    continue;
                };
                let contents = cell.contents();
                if contents.is_empty() {
                    dst.set_symbol(" ");
                } else {
                    dst.set_symbol(contents);
                }
                let mut style = RtStyle::default()
                    .fg(map_color(cell.fgcolor(), true))
                    .bg(map_color(cell.bgcolor(), false));
                let mut modifier = RtModifier::empty();
                if cell.bold() {
                    modifier |= RtModifier::BOLD;
                }
                if cell.italic() {
                    modifier |= RtModifier::ITALIC;
                }
                if cell.underline() {
                    modifier |= RtModifier::UNDERLINED;
                }
                if cell.dim() {
                    modifier |= RtModifier::DIM;
                }
                if cell.inverse() {
                    modifier |= RtModifier::REVERSED;
                }
                style = style.add_modifier(modifier);
                dst.set_style(style);
            }
        }
    }
}

pub fn rasterize_system(
    mut q: Query<(
        &mut TerminalScreen,
        &TerminalTexture,
        &TerminalSize,
        &mut TerminalRenderer,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut screen, tex, size, mut renderer, material_handle) in q.iter_mut() {
        if !screen.dirty {
            continue;
        }
        let rows = size.rows;
        let cols = size.cols;

        let (cur_row, cur_col) = screen.parser.screen().cursor_position();
        let hide_cursor = screen.parser.screen().hide_cursor();

        let draw_res = {
            let vt_screen = screen.parser.screen();
            let widget = VtScreenWidget {
                screen: vt_screen,
                rows,
                cols,
            };
            renderer.terminal.draw(|frame| {
                let area = Rect::new(0, 0, cols, rows);
                frame.render_widget(widget, area);
                if !hide_cursor {
                    frame.set_cursor_position((cur_col, cur_row));
                }
            })
        };
        if draw_res.is_err() {
            continue;
        }

        let backend = renderer.terminal.backend();
        let width = backend.get_pixmap_width() as u32;
        let height = backend.get_pixmap_height() as u32;
        let rgba = backend.get_pixmap_data_as_rgba();

        renderer.frames += 1;
        if renderer.frames <= 3 || renderer.frames % 60 == 0 {
            let mut nonzero = 0usize;
            for px in rgba.chunks_exact(4) {
                if px[0] != 0 || px[1] != 0 || px[2] != 0 {
                    nonzero += 1;
                }
            }
            eprintln!(
                "[clankertopia] rasterize frame={} {}x{} bytes={} nonzero_px={}",
                renderer.frames,
                width,
                height,
                rgba.len(),
                nonzero
            );
            if renderer.frames == 2 {
                let rgb: Vec<u8> = rgba
                    .chunks_exact(4)
                    .flat_map(|c| [c[0], c[1], c[2]])
                    .collect();
                let header = format!("P6\n{width} {height}\n255\n");
                if let Ok(mut f) = std::fs::File::create("/tmp/clankertopia_frame.ppm") {
                    use std::io::Write;
                    let _ = f.write_all(header.as_bytes());
                    let _ = f.write_all(&rgb);
                    eprintln!("[clankertopia] wrote /tmp/clankertopia_frame.ppm");
                }
            }
        }

        if let Some(image) = images.get_mut(&tex.0) {
            let need_resize = image.texture_descriptor.size.width != width
                || image.texture_descriptor.size.height != height;
            if need_resize {
                image.resize(Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                });
            }
            image.data = Some(rgba);
        }

        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color_texture = Some(tex.0.clone());
        }

        screen.dirty = false;
    }
}

pub fn resize_terminal_system(
    mut events: MessageReader<TerminalResize>,
    mut q: Query<(
        &mut TerminalSize,
        &mut TerminalScreen,
        &PtyHandle,
        Option<&mut TerminalRenderer>,
    )>,
) {
    for ev in events.read() {
        if let Ok((mut size, mut screen, pty, renderer)) = q.get_mut(ev.entity) {
            if size.cols == ev.cols && size.rows == ev.rows {
                continue;
            }
            size.cols = ev.cols;
            size.rows = ev.rows;
            screen.parser.screen_mut().set_size(ev.rows, ev.cols);
            let _ = pty.0.resize_tx.send((ev.cols, ev.rows));
            if let Some(mut renderer) = renderer {
                // Pixmap is sized cols*char_width x rows*char_height inside
                // SoftBackend, so we rebuild it whenever cell counts change.
                let font_size = renderer.font_size;
                let backend = SoftBackend::<CosmicText>::new(
                    ev.cols,
                    ev.rows,
                    font_size,
                    NERD_FONT_BYTES,
                );
                let mut new_terminal = Terminal::new(backend).expect("ratatui terminal");
                let _ = new_terminal.clear();
                renderer.terminal = new_terminal;
                renderer.frames = 0;
            }
            screen.dirty = true;
        }
    }
}

fn map_color(c: vt100::Color, fg: bool) -> RtColor {
    match c {
        vt100::Color::Default => {
            if fg {
                RtColor::Rgb(220, 220, 220)
            } else {
                RtColor::Rgb(8, 10, 14)
            }
        }
        vt100::Color::Idx(i) => indexed_to_color(i),
        vt100::Color::Rgb(r, g, b) => RtColor::Rgb(r, g, b),
    }
}

fn indexed_to_color(i: u8) -> RtColor {
    const PALETTE: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 49, 49),
        (13, 188, 121),
        (229, 229, 16),
        (36, 114, 200),
        (188, 63, 188),
        (17, 168, 205),
        (229, 229, 229),
        (102, 102, 102),
        (241, 76, 76),
        (35, 209, 139),
        (245, 245, 67),
        (59, 142, 234),
        (214, 112, 214),
        (41, 184, 219),
        (255, 255, 255),
    ];
    if (i as usize) < PALETTE.len() {
        let (r, g, b) = PALETTE[i as usize];
        RtColor::Rgb(r, g, b)
    } else if (16..=231).contains(&i) {
        let n = i - 16;
        let r = n / 36;
        let g = (n % 36) / 6;
        let b = n % 6;
        let v = |c: u8| if c == 0 { 0 } else { 55 + c * 40 };
        RtColor::Rgb(v(r), v(g), v(b))
    } else {
        let g = 8 + (i - 232) * 10;
        RtColor::Rgb(g, g, g)
    }
}
