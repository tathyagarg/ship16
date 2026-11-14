use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::{BufWriter, Write},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
};

use crate::{
    constants::*,
    emulator::Emulator,
    instructions::{Instr, decode},
};

pub(crate) mod constants;
pub(crate) mod cpu;
pub(crate) mod emulator;
pub(crate) mod input;
pub(crate) mod instructions;
pub(crate) mod interrupt;
pub(crate) mod memory;
pub(crate) mod ppu;
pub(crate) mod terminal;
pub(crate) mod timer;

const FRAME_DURATION: Duration = Duration::from_micros(CYCLES_PER_FRAME);

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> std::io::Result<Self> {
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

fn restore_terminal() {
    let _ = execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show,
    );
    let _ = crossterm::terminal::disable_raw_mode();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    let mut emulator = Emulator::new();
    emulator.start();
    emulator.load_rom(
        &[
            0x3000, // 0900 MOVE_TO #000
            0x7000, // 0902 MOV R0, 0
            0x7F01, // 0904 MOV RF, 1
            0x7E3F, // 0906 MOV RE, 63
            0x7D40, // 0908 MOV RD, 64
            0xD001, // 090A RAND R1
            0x2101, // 090C DRWR R1
            0x610E, // 090E SKP<NE> R0, RE
            0x4101, // 0910 INCY 1
            0x4001, // 0912 INCX 1
            0xC00F, // 0914 ADD R0, RF
            0xC40D, // 0916 REM R0, RD
            0x590A, // 0918 JUMP 090A
        ]
        .map(|word: u16| word.to_le_bytes())
        .concat(),
    )?;

    {
        let _guard = TerminalGuard::new()?;
        frame_loop(&mut emulator)?;
        if !emulator.is_running() {
            restore_terminal();
        }
    }

    Ok(())
}

fn frame_loop(emu: &mut Emulator) -> Result<(), Box<dyn std::error::Error>> {
    let mut frame = String::with_capacity(16 * 1024);
    let mut next = Instant::now();

    let truecolor = std::env::var("COLORTERM")
        .map(|v| v.contains("truecolor") || v.contains("24bit"))
        .unwrap_or(false);

    while emu.is_running() {
        if !terminal::pump_events(emu)? {
            break;
        }

        emu.run_frame();
        terminal::draw(
            &mut std::io::stdout().lock(),
            emu.vram(),
            &mut frame,
            truecolor,
        )?;

        next += FRAME_DURATION;
        match next.checked_duration_since(Instant::now()) {
            Some(duration) => std::thread::sleep(duration),
            None => next = Instant::now(),
        }
    }

    Ok(())
}
