use clap::Parser;
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

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    rom: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(short, long, value_name = "ADDRESS")]
    inspect: Option<u16>,
}

const FRAME_DURATION: Duration = Duration::from_micros(CYCLES_PER_FRAME);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Err(msg) = terminal::check_size(args.debug) {
        eprintln!("Error: {msg}");
        std::process::exit(1);
    }

    let rom_data = std::fs::read(&args.rom)?;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        terminal::restore_terminal();
        default_hook(info);
    }));

    let mut emulator = Emulator::new();
    emulator.start();
    emulator.load_rom(&rom_data)?;

    {
        let _guard = terminal::TerminalGuard::new()?;
        frame_loop(&mut emulator, args.debug, args.inspect)?;
        if !emulator.is_running() {
            terminal::restore_terminal();
        }
    }

    Ok(())
}

fn frame_loop(
    emu: &mut Emulator,
    debug: bool,
    inspect: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
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
            emu,
            debug,
            inspect,
        )?;

        next += FRAME_DURATION;
        match next.checked_duration_since(Instant::now()) {
            Some(duration) => std::thread::sleep(duration),
            None => next = Instant::now(),
        }
    }

    Ok(())
}
