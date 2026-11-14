use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::{BufWriter, Write},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute, terminal,
};

use crate::{
    constants::SCREEN_WIDTH,
    instructions::{Instr, decode},
};

pub(crate) mod constants;
pub(crate) mod cpu;
pub(crate) mod input;
pub(crate) mod instructions;
pub(crate) mod interrupt;
pub(crate) mod memory;
pub(crate) mod ppu;
pub(crate) mod timer;

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(
            std::io::stdout(),
            terminal::EnterAlternateScreen,
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
        terminal::LeaveAlternateScreen,
        crossterm::cursor::Show,
    );
    let _ = terminal::disable_raw_mode();
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

fn render(
    vram: &[u8],
    buffer: &mut String,
    truecolor: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    buffer.clear();
    buffer.push_str("\x1b[H┌");
    buffer.push_str(&"─".repeat(64));
    buffer.push_str("┐\r\n");

    for y in (0..64).step_by(2) {
        buffer.push_str("│");
        let mut last: Option<(u8, u8)> = None;
        for x in 0..64 {
            let fg = vram[y * 64 + x];
            let bg = vram[(y + 1) * 64 + x];
            if last != Some((fg, bg)) {
                if truecolor {
                    let [fr, fg, fb] = PALETTE[fg as usize];
                    let [br, bg, bb] = PALETTE[bg as usize];
                    buffer.push_str(&format!(
                        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
                        fr, fg, fb, br, bg, bb
                    ));
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

    buffer.push_str("└");
    buffer.push_str(&"─".repeat(64));
    buffer.push_str("┘\r\n");

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

pub struct Emulator {
    cpu: cpu::Cpu,
    memory: memory::Memory,
    timer: timer::Timer,
    ppu: ppu::Ppu,
    input: input::Input,
    logger: Option<BufWriter<File>>,

    interrupt_controller: interrupt::InterruptController,
    running: bool,

    frame: String,
    truecolor: bool,
}

macro_rules! trace {
    ($emu:expr, $($arg:tt)*) => {
        if let Some(w) = $emu.logger.as_mut() {
            let _ = writeln!(w, $($arg)*);
        }
    };
}

impl Emulator {
    pub fn new() -> Self {
        let logger = std::env::var_os("SHIP8_LOG")
            .and_then(|path| File::create(path).ok())
            .map(BufWriter::new);

        Emulator {
            cpu: cpu::Cpu::new(),
            memory: memory::Memory::new(),
            timer: timer::Timer::new(),
            ppu: ppu::Ppu::new(),
            input: input::Input::new(),
            logger,
            interrupt_controller: interrupt::InterruptController::new(),
            running: false,

            frame: String::with_capacity(16 * 1024),
            truecolor: std::env::var("COLORTERM")
                .map(|v| v.contains("truecolor") || v.contains("24bit"))
                .unwrap_or(false),
        }
    }

    pub fn log(&mut self, message: &str) {
        if let Some(logger) = &mut self.logger {
            use std::io::Write;
            let _ = writeln!(logger, "{}", message);
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        if self.timer.tick_n(cycles as u16) {
            self.interrupt_controller
                .request_interrupt(interrupt::INTERRUPT_TIMER);
        }

        if self.ppu.tick_n(cycles as u64) {
            self.interrupt_controller
                .request_interrupt(interrupt::INTERRUPT_VBLANK);
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn clear_buffer(&mut self) {
        self.memory.clear_buffer();
    }

    pub fn draw_color(&mut self, color: u8) {
        self.memory.draw_color(color, self.ppu.x, self.ppu.y);
    }

    pub fn draw_sprite(&mut self, sprite_index: u8) {
        let sprite_data = self
            .memory
            .read_sprite((self.cpu.index + (64 * sprite_index as u16)) as usize);

        for (i, &color) in sprite_data.iter().enumerate() {
            let x = (self.ppu.x + (i as u8 % 8)) % 64;
            let y = (self.ppu.y + (i as u8 / 8)) % 64;
            self.memory.draw_color(color, x, y);
        }
    }

    pub fn execute_instruction(&mut self) -> u32 {
        const W: u8 = SCREEN_WIDTH as u8;

        let pc = self.cpu.pc;
        let word = self.memory.read_u16(pc);
        self.cpu.pc = self.cpu.pc.wrapping_add(2);

        let instr = match decode(word) {
            Ok(instr) => instr,
            Err(e) => {
                self.log(&format!("Error decoding instruction at {:04X}: {}", pc, e));
                self.running = false;
                return 1;
            }
        };
        trace!(self, "[{pc:04X}] {instr:?}");
        let mut skip = false;

        match instr {
            Instr::Nop => {}
            Instr::Clear => self.memory.clear_buffer(),
            Instr::DrawColor(c) => self.draw_color(c),
            Instr::DrawColorReg(r) => self.draw_color(self.cpu.registers[r as usize] as u8),
            Instr::DrawSprite(i) => self.draw_sprite(i),
            Instr::DrawSpriteReg(r) => self.draw_sprite(self.cpu.registers[r as usize] as u8),
            Instr::MoveTo(p) => self.ppu.move_to_decode(p),
            Instr::AddX(v) => self.ppu.x = self.ppu.x.wrapping_add(v) % W,
            Instr::SubX(v) => self.ppu.x = self.ppu.x.wrapping_sub(v) % W,
            Instr::AddY(v) => self.ppu.y = self.ppu.y.wrapping_add(v) % W,
            Instr::SubY(v) => self.ppu.y = self.ppu.y.wrapping_sub(v) % W,
            Instr::SetXReg(r) => self.ppu.x = (self.cpu.registers[r as usize] % u16::from(W)) as u8,
            Instr::SetYReg(r) => self.ppu.y = (self.cpu.registers[r as usize] % u16::from(W)) as u8,
            Instr::Jump(a) => self.cpu.pc = a,
            Instr::SkipIf {
                cond,
                reg_x: x,
                reg_y: y,
            } => {
                skip = cond.eval(
                    self.cpu.registers[x as usize],
                    self.cpu.registers[y as usize],
                );
            }
            Instr::SetReg { reg, value } => self.cpu.registers[reg as usize] = value as u16,
            Instr::Copy {
                reg_x: src,
                reg_y: dst,
            } => {
                self.cpu.registers[dst as usize] = self.cpu.registers[src as usize];
            }
            Instr::Call(a) => {
                let ret = self.cpu.pc;
                match self.cpu.push_u16(&mut self.memory, ret) {
                    Ok(()) => self.cpu.pc = a,
                    Err(e) => {
                        trace!(self, "[{pc:04X}] {e:?}");
                        self.running = false;
                    }
                }
            }
            Instr::Return => match self.cpu.pop_u16(&self.memory) {
                Ok(a) => self.cpu.pc = a,
                Err(e) => {
                    trace!(self, "[{pc:04X}] {e:?}");
                    self.running = false;
                }
            },
            Instr::SkipIfKey(k) => skip = self.input.is_pressed(k),
            Instr::Alu {
                op,
                reg_x: x,
                reg_y: y,
            } => {
                let r = op.eval(
                    self.cpu.registers[x as usize],
                    self.cpu.registers[y as usize],
                );
                self.cpu.registers[x as usize] = r;
            }
            Instr::Random(r) => self.cpu.registers[r as usize] = rand::random::<u8>() as u16,
            Instr::SetIndex(a) => self.cpu.index = a,
            Instr::Halt => self.running = false,
        }

        if skip {
            self.cpu.pc = self.cpu.pc.wrapping_add(2);
        }

        instr.base_cycles() + (skip as u32)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.running {
            if event::poll(Duration::ZERO)? {
                if let Event::Key(key_event) = event::read()? {
                    match key_event.kind {
                        KeyEventKind::Press => {
                            if key_event.code == KeyCode::Esc {
                                self.running = false;
                                continue;
                            }

                            if let Some(keycode) = decode_keycode(key_event.code) {
                                self.input.key_down(keycode);
                                self.interrupt_controller
                                    .request_interrupt(interrupt::INTERRUPT_INPUT);
                            }
                        }
                        KeyEventKind::Release => {
                            if let Some(keycode) = decode_keycode(key_event.code) {
                                self.input.key_up(keycode);
                            }
                        }
                        KeyEventKind::Repeat => {}
                    }
                }
            }

            let cycles = self.execute_instruction();

            self.tick(cycles);

            if self.interrupt_controller.is_any_interrupt_pending() {
                while let Some(interrupt) = self.interrupt_controller.get_pending_interrupt() {
                    if interrupt == 0 {
                        let vram = self.memory.vram();
                        render(vram, &mut self.frame, self.truecolor)?;

                        let mut stdout = std::io::stdout().lock();
                        stdout.write_all(self.frame.as_bytes())?;
                        stdout.flush()?;
                    } else if interrupt == 1 {
                        // println!("TIMER interrupt triggered");
                    } else if interrupt == 2 {
                        // println!("INPUT interrupt triggered");
                    }

                    self.interrupt_controller.clear_interrupt(1 << interrupt);
                }
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    let mut emulator = Emulator::new();
    emulator.start();
    emulator.memory.load_rom(&[
        0x00, 0x30, 0x00, 0x10, 0x00, 0x70, 0x05, 0x7F, 0x01, 0x7E, 0x00, 0xE0, 0x00, 0x2E, 0x08,
        0x40, 0x0E, 0xC0, 0x0F, 0x60, 0x0C, 0x59, 0x00, 0x00, 0x16, 0x59,
    ])?;

    let _guard = TerminalGuard::new()?;
    emulator.run()?;

    Ok(())
}
