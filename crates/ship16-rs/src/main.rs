use std::time::{Duration, Instant};

use crate::emulator::Emulator;
use isa::constants::*;

pub(crate) mod cpu;
pub(crate) mod emulator;
pub(crate) mod eval;
pub(crate) mod input;
pub(crate) mod memory;
pub(crate) mod ppu;
pub(crate) mod terminal;
pub(crate) mod timer;

const FRAME_DURATION: Duration = Duration::from_micros(CYCLES_PER_FRAME);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        terminal::restore_terminal();
        default_hook(info);
    }));

    let mut emulator = Emulator::new();
    emulator.start();
    emulator.load_rom(&[
        0x00, 0x30, 0x00, 0x10, 0x00, 0xF3, 0x00, 0x00, 0x0F, 0xF3, 0x01, 0x00, 0x0E, 0xF3, 0x3F,
        0x00, 0x0D, 0xF3, 0x40, 0x00, 0x00, 0xF2, 0x28, 0x09, 0x02, 0x13, 0x04, 0x12, 0x00, 0x14,
        0x01, 0xD0, 0x00, 0xF1, 0x34, 0x09, 0x00, 0xF0, 0x1E, 0x09, 0x01, 0xF3, 0x03, 0x00, 0x00,
        0xF1, 0x34, 0x09, 0x00, 0xF0, 0x2C, 0x09, 0x01, 0x21, 0x0E, 0x61, 0x01, 0x41, 0x01, 0x40,
        0x0F, 0xC0, 0x0D, 0xC4, 0x00, 0xA0,
    ])?;

    {
        let _guard = terminal::TerminalGuard::new()?;
        frame_loop(&mut emulator)?;
        if !emulator.is_running() {
            terminal::restore_terminal();
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
