// Standalone test: feed bytes through vt100 -> soft_ratatui and write the
// resulting pixmap to /tmp/raster_probe.ppm. If the PPM is all-zero / all-bg,
// the rasterizer pipeline itself is broken (independent of Bevy/GPU).

use ratatui::Terminal;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color as RtColor, Modifier as RtModifier, Style as RtStyle};
use ratatui::widgets::Widget;
use soft_ratatui::{CosmicText, SoftBackend};

static FONT_BYTES: &[u8] =
    include_bytes!("../assets/fonts/NerdFontMono-Regular.ttf");
use std::fs::File;
use std::io::Write;

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
                let style = RtStyle::default()
                    .fg(RtColor::Rgb(220, 220, 220))
                    .bg(RtColor::Rgb(8, 10, 14))
                    .add_modifier(if cell.bold() {
                        RtModifier::BOLD
                    } else {
                        RtModifier::empty()
                    });
                dst.set_style(style);
            }
        }
    }
}

fn main() {
    let cols = 100u16;
    let rows = 30u16;

    let mut parser = vt100::Parser::new(rows, cols, 0);
    parser.process(b"hello clankertopia\r\n");
    // Nerd-Font glyphs in the Private Use Area:
    //   U+F121 (devicon)   U+E0B0 (powerline arrow)   U+F07B (folder)
    parser.process("nerd icons:  \u{f121}  \u{e0b0}  \u{f07b}\r\n".as_bytes());
    parser.process("box drawing: \u{2554}\u{2550}\u{2557}\r\n".as_bytes());
    parser.process(b"$ ls -la\r\n");

    let backend = SoftBackend::<CosmicText>::new(cols, rows, 16, FONT_BYTES);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.clear().expect("clear");

    let cur = parser.screen().cursor_position();
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, cols, rows);
            frame.render_widget(
                VtScreenWidget {
                    screen: parser.screen(),
                    rows,
                    cols,
                },
                area,
            );
            frame.set_cursor_position((cur.1, cur.0));
        })
        .expect("draw");

    let backend = terminal.backend();
    let w = backend.get_pixmap_width() as u32;
    let h = backend.get_pixmap_height() as u32;
    let rgba = backend.get_pixmap_data_as_rgba();
    let rgb = backend.get_pixmap_data();
    println!(
        "pixmap: {}x{}  rgba_bytes={}  rgb_bytes={}",
        w,
        h,
        rgba.len(),
        rgb.len()
    );

    // Quick histogram of non-zero pixels to detect "all black" output.
    let mut zero = 0usize;
    let mut nonzero = 0usize;
    let mut max_byte = 0u8;
    for px in rgba.chunks_exact(4) {
        if px[0] == 0 && px[1] == 0 && px[2] == 0 {
            zero += 1;
        } else {
            nonzero += 1;
        }
        max_byte = max_byte.max(px[0]).max(px[1]).max(px[2]);
    }
    println!(
        "pixels zero={zero} nonzero={nonzero} max_channel={max_byte}"
    );

    // Write a PPM (P6, RGB) we can eyeball with any image viewer.
    let mut f = File::create("/tmp/raster_probe.ppm").expect("create ppm");
    let header = format!("P6\n{w} {h}\n255\n");
    f.write_all(header.as_bytes()).unwrap();
    f.write_all(&rgb).unwrap();
    println!("wrote /tmp/raster_probe.ppm");
}
