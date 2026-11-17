use std::{fmt::Write, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
};

use crate::emulator::Emulator;

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> std::io::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide,
            crossterm::event::PushKeyboardEnhancementFlags(
                crossterm::event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

pub fn restore_terminal() {
    let _ = execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show,
    );
    let _ = crossterm::terminal::disable_raw_mode();
}

fn decode_keycode(keycode: KeyCode) -> Option<u8> {
    match keycode {
        KeyCode::Char(c) => {
            let c = c.to_ascii_lowercase();

            match c {
                '0'..='9' => Some(c as u8 - b'0'),
                'a'..='f' => Some(c as u8 - b'a' + 10),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn pump_events(emulator: &mut Emulator) -> std::io::Result<bool> {
    while event::poll(Duration::ZERO)? {
        let Event::Key(key) = event::read()? else {
            continue;
        };
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.kind {
            KeyEventKind::Press => {
                if key.code == KeyCode::Esc || (ctrl && key.code == KeyCode::Char('c')) {
                    return Ok(false);
                }
                if !ctrl && let Some(k) = decode_keycode(key.code) {
                    emulator.key_down(k);
                }
            }
            KeyEventKind::Release => {
                if let Some(k) = decode_keycode(key.code) {
                    emulator.key_up(k);
                }
            }
            KeyEventKind::Repeat => {}
        }
    }
    Ok(true)
}

fn render(
    vram: &[u8],
    buffer: &mut String,
    truecolor: bool,
    emu: &Emulator,
    debug: bool,
    inspect: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    buffer.clear();
    let title = " Ship-16 Emulator ";
    let padding = (64 - title.len()) / 2;

    buffer.push_str("\x1b[H┌");
    buffer.push_str(&"─".repeat(padding));
    buffer.push_str(title);
    buffer.push_str(&"─".repeat(64 - padding - title.len()));
    buffer.push_str("┐\r\n");

    for y in (0..64).step_by(2) {
        buffer.push('│');
        let mut last: Option<(u8, u8)> = None;
        for x in 0..64 {
            let fg = vram[y * 64 + x];
            let bg = vram[(y + 1) * 64 + x];
            if last != Some((fg, bg)) {
                if truecolor {
                    let [fr, fg, fb] = PALETTE[fg as usize];
                    let [br, bg, bb] = PALETTE[bg as usize];
                    write!(
                        buffer,
                        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
                        fr, fg, fb, br, bg, bb
                    )?;
                } else {
                    let _ = write!(
                        buffer,
                        "\x1b[38;5;{}m\x1b[48;5;{}m",
                        to_ansi256(fg),
                        to_ansi256(bg)
                    );
                }

                last = Some((fg, bg));
            }
            buffer.push('▀');
        }
        buffer.push_str("\x1b[0m│\r\n");
    }

    buffer.push('└');
    buffer.push_str(&"─".repeat(64));
    buffer.push_str("┘\r\n");

    if debug {
        push_debug_panel(buffer, emu)?;
    }

    if let Some(addr) = inspect {
        push_inspect_panel(buffer, emu, addr)?;
    }

    Ok(())
}

const fn build_palette() -> [[u8; 3]; 256] {
    let mut palette = [[0u8; 3]; 256];
    let mut i = 0;
    while i < 256 {
        let r = ((i >> 5) & 0x07) * 255 / 7;
        let g = ((i >> 2) & 0x07) * 255 / 7;
        let b = (i & 0x03) * 255 / 3;
        palette[i] = [r as u8, g as u8, b as u8];
        i += 1;
    }

    palette
}

static PALETTE: [[u8; 3]; 256] = build_palette();

fn to_ansi256(c: u8) -> u8 {
    let [r, g, b] = PALETTE[c as usize];
    let q = |v: u8| ((v as u16 * 5 + 127) / 255) as u8;

    16 + q(r) * 36 + q(g) * 6 + q(b)
}

pub fn draw(
    out: &mut impl std::io::Write,
    vram: &[u8],
    buffer: &mut String,
    truecolor: bool,
    emu: &Emulator,
    debug: bool,
    inspect: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    render(vram, buffer, truecolor, emu, debug, inspect)?;
    out.write_all(buffer.as_bytes())?;
    out.flush()?;
    Ok(())
}
const MIN_COLS: u16 = 66;
const SCREEN_ROWS: u16 = 35;
const DEBUG_ROWS: u16 = 9;

pub fn required_size(debug: bool) -> (u16, u16) {
    (MIN_COLS, SCREEN_ROWS + if debug { DEBUG_ROWS } else { 0 })
}

pub fn check_size(debug: bool) -> Result<(), String> {
    let (cols, rows) =
        crossterm::terminal::size().map_err(|e| format!("cannot read terminal size: {e}"))?;
    let (need_cols, need_rows) = required_size(debug);
    if cols < need_cols || rows < need_rows {
        return Err(format!(
            "terminal is {cols}x{rows}; need at least {need_cols}x{need_rows}"
        ));
    }
    Ok(())
}

fn push_debug_panel(buffer: &mut String, emu: &Emulator) -> std::fmt::Result {
    buffer.push_str("Registers:\r\n");
    for (i, reg) in emu.regs().iter().enumerate() {
        write!(buffer, "R{:01X}: {:04X}  ", i, reg)?;
        if i % 4 == 3 {
            buffer.push_str("\r\n");
        }
    }
    buffer.push_str("\r\nTimer:\r\n");
    write!(
        buffer,
        "Counter: {:04X}  Reload: {:04X}  PSA: {:08X}\r\n",
        emu.timer().counter,
        emu.timer().reload_to,
        emu.timer().prescaler_accumulator
    )?;
    write!(
        buffer,
        "Enabled: {}  Periodic: {}\r\n",
        emu.timer().enabled,
        emu.timer().periodic
    )
}

fn push_inspect_panel(buffer: &mut String, emu: &Emulator, addr: u16) -> std::fmt::Result {
    if let Some(bytes) = emu.read_bytes(addr, 128) {
        buffer.push_str(&format!("\r\nMemory at {:04X}:\r\n", addr));
        for (i, chunk) in bytes.chunks(16).enumerate() {
            write!(buffer, "{:04X}: ", addr + (i * 16) as u16)?;
            for byte in chunk {
                write!(buffer, "{:02X} ", byte)?;
            }
            buffer.push_str("\r\n");
        }
    } else {
        buffer.push_str(&format!("Memory at {:04X}: <out of bounds>\r\n", addr));
    }

    Ok(())
}
