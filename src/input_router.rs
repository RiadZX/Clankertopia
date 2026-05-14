use std::collections::HashMap;

use bevy::ecs::message::MessageReader;
use bevy::ecs::message::MessageWriter;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::ButtonState;
use bevy::prelude::*;

use crate::clanker_communication::PromptSubmitted;
use crate::game_state::GameState;
use crate::terminal::systems::TerminalRenderer;
use crate::terminal::{PtyHandle, TerminalScreen};

pub struct InputRouterPlugin;

impl Plugin for InputRouterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (input_router_system, scroll_system));
    }
}

#[derive(Default)]
pub struct ModifierState {
    ctrl: bool,
    alt: bool,
    shift: bool,
}

const FONT_SIZE_MIN: i32 = 8;
const FONT_SIZE_MAX: i32 = 40;
const FONT_SIZE_STEP: i32 = 2;
const FONT_SIZE_DEFAULT: i32 = 16;

pub fn input_router_system(
    mut key_events: MessageReader<KeyboardInput>,
    mut modifiers: Local<ModifierState>,
    mut line_buffers: Local<HashMap<Entity, String>>,
    state: Res<GameState>,
    ptys: Query<&PtyHandle>,
    mut renderers: Query<(&mut TerminalRenderer, &mut TerminalScreen)>,
    mut prompts: MessageWriter<PromptSubmitted>,
) {
    let focused = state.focused_entity();

    for ev in key_events.read() {
        match ev.key_code {
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                modifiers.ctrl = ev.state == ButtonState::Pressed;
                continue;
            }
            KeyCode::AltLeft | KeyCode::AltRight => {
                modifiers.alt = ev.state == ButtonState::Pressed;
                continue;
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                modifiers.shift = ev.state == ButtonState::Pressed;
                continue;
            }
            _ => {}
        }

        if ev.state != ButtonState::Pressed {
            continue;
        }
        if matches!(ev.key_code, KeyCode::Escape) {
            continue;
        }

        let Some(focused_entity) = focused else {
            continue;
        };

        if modifiers.ctrl {
            let delta = match ev.key_code {
                KeyCode::Equal | KeyCode::NumpadAdd => Some(FONT_SIZE_STEP),
                KeyCode::Minus | KeyCode::NumpadSubtract => Some(-FONT_SIZE_STEP),
                KeyCode::Digit0 | KeyCode::Numpad0 => {
                    set_font_size(focused_entity, FONT_SIZE_DEFAULT, &mut renderers);
                    continue;
                }
                _ => None,
            };
            if let Some(d) = delta {
                let cur = renderers
                    .get(focused_entity)
                    .map(|(r, _)| r.font_size)
                    .unwrap_or(FONT_SIZE_DEFAULT);
                let new_size = (cur + d).clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
                set_font_size(focused_entity, new_size, &mut renderers);
                continue;
            }
        }

        let Ok(pty) = ptys.get(focused_entity) else {
            continue;
        };

        let line = line_buffers.entry(focused_entity).or_default();
        match ev.key_code {
            KeyCode::Backspace => {
                line.pop();
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let prompt = line.trim();
                if !prompt.is_empty() {
                    prompts.write(PromptSubmitted {
                        source: focused_entity,
                        prompt: prompt.to_string(),
                    });
                }
                line.clear();
            }
            _ => {
                if !modifiers.ctrl && !modifiers.alt {
                    if let Some(text) = ev.text.as_ref() {
                        let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                        if !filtered.is_empty() {
                            line.push_str(&filtered);
                        }
                    }
                }
            }
        }

        if let Some(bytes) = encode_key(ev, modifiers.ctrl, modifiers.alt) {
            let _ = pty.0.input_tx.send(bytes);
            // Typing always returns the view to the live bottom line.
            if let Ok((_, mut screen)) = renderers.get_mut(focused_entity) {
                if screen.parser.screen().scrollback() != 0 {
                    screen.parser.screen_mut().set_scrollback(0);
                    screen.dirty = true;
                }
            }
        }
    }
}

fn set_font_size(
    entity: Entity,
    font_size: i32,
    renderers: &mut Query<(&mut TerminalRenderer, &mut TerminalScreen)>,
) {
    let Ok((mut renderer, mut screen)) = renderers.get_mut(entity) else {
        return;
    };
    if renderer.font_size == font_size {
        return;
    }
    eprintln!("[clankertopia] font_size={font_size}");
    renderer.font_size = font_size;
    renderer.terminal.backend_mut().set_font_size(font_size);
    renderer.frames = 0;
    screen.dirty = true;
}

fn encode_key(ev: &KeyboardInput, ctrl: bool, alt: bool) -> Option<Vec<u8>> {
    let prefix_alt = |mut v: Vec<u8>| {
        if alt {
            let mut out = vec![0x1b];
            out.append(&mut v);
            out
        } else {
            v
        }
    };

    if ctrl {
        if let Some(byte) = ctrl_byte(ev.key_code) {
            return Some(prefix_alt(vec![byte]));
        }
    }

    match ev.key_code {
        KeyCode::Enter | KeyCode::NumpadEnter => return Some(prefix_alt(vec![b'\r'])),
        KeyCode::Backspace => return Some(prefix_alt(vec![0x7f])),
        KeyCode::Tab => return Some(prefix_alt(vec![b'\t'])),
        KeyCode::Space => return Some(prefix_alt(vec![b' '])),
        KeyCode::ArrowUp => return Some(prefix_alt(b"\x1b[A".to_vec())),
        KeyCode::ArrowDown => return Some(prefix_alt(b"\x1b[B".to_vec())),
        KeyCode::ArrowRight => return Some(prefix_alt(b"\x1b[C".to_vec())),
        KeyCode::ArrowLeft => return Some(prefix_alt(b"\x1b[D".to_vec())),
        KeyCode::Home => return Some(prefix_alt(b"\x1b[H".to_vec())),
        KeyCode::End => return Some(prefix_alt(b"\x1b[F".to_vec())),
        KeyCode::PageUp => return Some(prefix_alt(b"\x1b[5~".to_vec())),
        KeyCode::PageDown => return Some(prefix_alt(b"\x1b[6~".to_vec())),
        KeyCode::Insert => return Some(prefix_alt(b"\x1b[2~".to_vec())),
        KeyCode::Delete => return Some(prefix_alt(b"\x1b[3~".to_vec())),
        KeyCode::F1 => return Some(prefix_alt(b"\x1bOP".to_vec())),
        KeyCode::F2 => return Some(prefix_alt(b"\x1bOQ".to_vec())),
        KeyCode::F3 => return Some(prefix_alt(b"\x1bOR".to_vec())),
        KeyCode::F4 => return Some(prefix_alt(b"\x1bOS".to_vec())),
        KeyCode::F5 => return Some(prefix_alt(b"\x1b[15~".to_vec())),
        KeyCode::F6 => return Some(prefix_alt(b"\x1b[17~".to_vec())),
        KeyCode::F7 => return Some(prefix_alt(b"\x1b[18~".to_vec())),
        KeyCode::F8 => return Some(prefix_alt(b"\x1b[19~".to_vec())),
        KeyCode::F9 => return Some(prefix_alt(b"\x1b[20~".to_vec())),
        KeyCode::F10 => return Some(prefix_alt(b"\x1b[21~".to_vec())),
        KeyCode::F11 => return Some(prefix_alt(b"\x1b[23~".to_vec())),
        KeyCode::F12 => return Some(prefix_alt(b"\x1b[24~".to_vec())),
        _ => {}
    }

    if let Some(text) = ev.text.as_ref() {
        if !text.is_empty() {
            return Some(prefix_alt(text.as_bytes().to_vec()));
        }
    }
    if let Key::Character(s) = &ev.logical_key {
        if !s.is_empty() {
            return Some(prefix_alt(s.as_bytes().to_vec()));
        }
    }

    None
}

fn ctrl_byte(key: KeyCode) -> Option<u8> {
    match key {
        KeyCode::KeyA => Some(0x01),
        KeyCode::KeyB => Some(0x02),
        KeyCode::KeyC => Some(0x03),
        KeyCode::KeyD => Some(0x04),
        KeyCode::KeyE => Some(0x05),
        KeyCode::KeyF => Some(0x06),
        KeyCode::KeyG => Some(0x07),
        KeyCode::KeyH => Some(0x08),
        KeyCode::KeyI => Some(0x09),
        KeyCode::KeyJ => Some(0x0a),
        KeyCode::KeyK => Some(0x0b),
        KeyCode::KeyL => Some(0x0c),
        KeyCode::KeyM => Some(0x0d),
        KeyCode::KeyN => Some(0x0e),
        KeyCode::KeyO => Some(0x0f),
        KeyCode::KeyP => Some(0x10),
        KeyCode::KeyQ => Some(0x11),
        KeyCode::KeyR => Some(0x12),
        KeyCode::KeyS => Some(0x13),
        KeyCode::KeyT => Some(0x14),
        KeyCode::KeyU => Some(0x15),
        KeyCode::KeyV => Some(0x16),
        KeyCode::KeyW => Some(0x17),
        KeyCode::KeyX => Some(0x18),
        KeyCode::KeyY => Some(0x19),
        KeyCode::KeyZ => Some(0x1a),
        KeyCode::Space => Some(0x00),
        KeyCode::BracketLeft => Some(0x1b),
        KeyCode::Backslash => Some(0x1c),
        KeyCode::BracketRight => Some(0x1d),
        _ => None,
    }
}

/// One line per "notch" of a discrete wheel event; pixel devices (touchpads)
/// accumulate by this many pixels before scrolling a line.
const PIXELS_PER_LINE: f32 = 28.0;
const PAGE_FRACTION: f32 = 0.9;

#[derive(Default)]
pub struct ScrollAccumulator {
    px: f32,
}

pub fn scroll_system(
    mut wheel: MessageReader<MouseWheel>,
    mut keys: MessageReader<KeyboardInput>,
    button_keys: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut acc: Local<ScrollAccumulator>,
    ptys: Query<&PtyHandle>,
    mut screens: Query<(&mut TerminalScreen, &crate::terminal::TerminalSize)>,
) {
    let Some(focused) = state.focused_entity() else {
        acc.px = 0.0;
        // Still drain so events don't pile up.
        wheel.read().for_each(|_| {});
        keys.read().for_each(|_| {});
        return;
    };

    let mut lines_delta: i32 = 0;

    // Mouse wheel.
    for ev in wheel.read() {
        let delta = match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => {
                acc.px += ev.y;
                let lines = (acc.px / PIXELS_PER_LINE).trunc();
                acc.px -= lines * PIXELS_PER_LINE;
                lines
            }
        };
        // Positive y = scroll up = show older lines.
        lines_delta += (delta * 3.0).round() as i32;
    }

    // Shift + PgUp / PgDn / Home / End for scrollback navigation by keyboard.
    let shift = button_keys.pressed(KeyCode::ShiftLeft) || button_keys.pressed(KeyCode::ShiftRight);

    let Ok((mut screen, size)) = screens.get_mut(focused) else {
        keys.read().for_each(|_| {});
        return;
    };
    let page = ((size.rows as f32) * PAGE_FRACTION).max(1.0) as i32;

    for ev in keys.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        if !shift {
            continue;
        }
        match ev.key_code {
            KeyCode::PageUp => lines_delta += page,
            KeyCode::PageDown => lines_delta -= page,
            KeyCode::Home => lines_delta += i32::MAX / 4,
            KeyCode::End => lines_delta = i32::MIN / 4,
            _ => {}
        }
    }

    if lines_delta == 0 {
        return;
    }

    let vt_screen = screen.parser.screen();
    if vt_screen.alternate_screen() {
        // Alt screen has no scrollback; emulate wheel as up/down arrow keys
        // (xterm-compatible behaviour for apps like less / vim / man).
        let Ok(pty) = ptys.get(focused) else {
            return;
        };
        let (seq, count) = if lines_delta > 0 {
            (b"\x1b[A".as_slice(), lines_delta)
        } else {
            (b"\x1b[B".as_slice(), -lines_delta)
        };
        let mut bytes = Vec::with_capacity(seq.len() * count.min(64) as usize);
        for _ in 0..count.min(64) {
            bytes.extend_from_slice(seq);
        }
        let _ = pty.0.input_tx.send(bytes);
        return;
    }

    // Normal screen: drive vt100's internal scrollback offset.
    let current = vt_screen.scrollback() as i32;
    let new = (current + lines_delta).max(0);
    screen.parser.screen_mut().set_scrollback(new as usize);
    screen.dirty = true;
}
