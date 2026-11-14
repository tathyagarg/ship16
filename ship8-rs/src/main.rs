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

pub(crate) mod cpu;
pub(crate) mod input;
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

fn decode_condition(cond: u8) -> fn(u16, u16) -> bool {
    match cond {
        0x00 => |x, y| x == y, // Equal
        0x01 => |x, y| x != y, // Not equal
        0x02 => |x, y| x < y,  // Less than
        0x03 => |x, y| x > y,  // Greater than
        0x04 => |x, y| x <= y, // Less than or equal
        0x05 => |x, y| x >= y, // Greater than or equal
        _ => |_x, _y| false,   // Always false
    }
}

fn decode_arithmetic(op: u8) -> (fn(u16, u16) -> u16, u32) {
    match op {
        0x00 => (|x, y| x.wrapping_add(y), 1),
        0x01 => (|x, y| x.wrapping_sub(y), 1),
        0x02 => (|x, y| x.wrapping_mul(y), 3),
        0x03 => (|x, y| if y == 0 { 0 } else { x.wrapping_div(y) }, 5),
        0x04 => (|x, y| if y == 0 { 0 } else { x.wrapping_rem(y) }, 5),
        0x05 => (|x, y| x & y, 1),
        0x06 => (|x, y| x | y, 1),
        0x07 => (|x, y| x ^ y, 1),
        0x08 => (|x, _y| !x, 1),
        0x09 => (|x, y| x.wrapping_shl(y as u32), 2),
        0x0A => (|x, y| x.wrapping_shr(y as u32), 2),
        0x0B => (|x, y| x.rotate_left(y as u32), 2),
        0x0C => (|x, y| x.rotate_right(y as u32), 2),
        _ => (|_x, _y| 0, 1), // Default case
    }
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
        let initial_pc = self.cpu.pc;
        let instruction = self.memory.fetch_next_instruction(&mut self.cpu.pc);
        match instruction {
            0x0000 => {
                // NOP
                trace!(self, "NOP [{:04X}]", initial_pc);
                1
            }
            0x1000 => {
                trace!(self, "CLEAR [{:04X}]", initial_pc);
                self.clear_buffer();
                4097
            }
            0x2000..=0x20FF => {
                let color = (instruction & 0x00FF) as u8;
                trace!(
                    self,
                    "DRAW_COLOR [{:04X}], color: {:02X}", initial_pc, color
                );
                self.draw_color(color);
                2
            }
            0x2100..=0x210F => {
                let x = instruction & 0x000F;
                let reg_x = self.cpu.registers[x as usize];
                self.log(
                    format!(
                        "DRAW_COLOR_REG [{:04X}], reg[{}]: {:02X}",
                        initial_pc, x, reg_x
                    )
                    .as_str(),
                );
                self.draw_color(reg_x as u8);
                2
            }
            0x2E00..=0x2E0F => {
                let reg_index = (instruction & 0x000F) as usize;
                let sprite_index = self.cpu.registers[reg_index] as u8;
                self.log(
                    format!(
                        "DRAW_SPRITE_REG [{:04X}], reg[{}]: {:02X}",
                        initial_pc, reg_index, sprite_index
                    )
                    .as_str(),
                );
                self.draw_sprite(sprite_index);
                66
            }
            0x2F00..=0x2FFF => {
                let sprite_index = (instruction & 0x00FF) as u8;
                self.log(
                    format!(
                        "DRAW_SPRITE [{:04X}], sprite_index: {:02X}",
                        initial_pc, sprite_index
                    )
                    .as_str(),
                );
                self.draw_sprite(sprite_index);
                66
            }
            0x3000..=0x3FFF => {
                self.log(
                    format!(
                        "MOVE_TO [{:04X}], pos: {:03X}",
                        initial_pc,
                        instruction & 0x0FFF
                    )
                    .as_str(),
                );
                self.ppu.move_to_decode(instruction & 0x0FFF);
                2
            }
            0x4000..=0x43FF => {
                let subinstr = (instruction >> 8) & 0x0F;
                let value = (instruction & 0x00FF) as u8;
                self.log(
                    format!(
                        "MOVE [{:04X}], subinstr: {:01X}, value: {:02X}",
                        initial_pc, subinstr, value
                    )
                    .as_str(),
                );
                match subinstr {
                    0x00 => self.ppu.x = self.ppu.x.wrapping_add(value) % 64,
                    0x01 => self.ppu.x = self.ppu.x.wrapping_sub(value) % 64,
                    0x02 => self.ppu.y = self.ppu.y.wrapping_add(value) % 64,
                    0x03 => self.ppu.y = self.ppu.y.wrapping_sub(value) % 64,
                    _ => unreachable!(),
                }

                2
            }
            0x4A00..=0x4A0F => {
                let reg_index = (instruction & 0x000F) as usize;
                let value = self.cpu.registers[reg_index];
                self.log(
                    format!(
                        "SET_X [{:04X}], reg[{}]: {:02X}",
                        initial_pc,
                        reg_index,
                        value % 64
                    )
                    .as_str(),
                );
                self.ppu.x = (value % 64) as u8;
                2
            }
            0x4A10..=0x4A1F => {
                let reg_index = (instruction & 0x000F) as usize;
                let value = self.cpu.registers[reg_index];

                self.log(
                    format!(
                        "SET_Y [{:04X}], reg[{}]: {:02X}",
                        initial_pc,
                        reg_index,
                        value % 64
                    )
                    .as_str(),
                );
                self.ppu.y = (value % 64) as u8;
                2
            }
            0x5000..=0x5FFF => {
                let address = instruction & 0x0FFF;
                trace!(self, "JUMP [{:04X}], address: {:03X}", initial_pc, address);
                self.cpu.pc = address;
                2
            }
            0x6000..=0x6FFF => {
                let cond = (instruction >> 8) & 0x0F;
                let x = (instruction >> 4) & 0x0F;
                let y = instruction & 0x0F;

                let cond_function = decode_condition(cond as u8);
                let rx = self.cpu.registers[x as usize];
                let ry = self.cpu.registers[y as usize];

                if cond_function(rx, ry) {
                    self.cpu.pc = self.cpu.pc.wrapping_add(2);
                    self.log(
                        format!(
                            "CONDITIONAL_JUMP [{:04X}], condition met, reg[{}]: {:02X}, reg[{}]: {:02X}",
                            initial_pc,
                            x,
                            rx,
                            y,
                            ry
                        )
                        .as_str(),
                    );
                    3
                } else {
                    self.log(
                        format!(
                            "CONDITIONAL_JUMP [{:04X}], condition not met, reg[{}]: {:02X}, reg[{}]: {:02X}",
                            initial_pc,
                            x,
                            rx,
                            y,
                            ry
                        )
                        .as_str(),
                    );
                    2
                }
            }
            0x7000..=0x7FFF => {
                let reg_index = ((instruction >> 8) & 0x0F) as usize;
                let value = instruction & 0x00FF;
                self.log(
                    format!(
                        "SET_REGISTER [{:04X}], reg[{}]: {:02X}",
                        initial_pc, reg_index, value
                    )
                    .as_str(),
                );
                self.cpu.registers[reg_index] = value;
                2
            }
            0x8000..=0x80FF => {
                let src_index = instruction & 0x0F;
                let dst_index = (instruction >> 4) & 0x0F;

                self.log(
                    format!(
                        "COPY_REGISTER [{:04X}], reg[{}]: {:02X} -> reg[{}]",
                        initial_pc, src_index, self.cpu.registers[src_index as usize], dst_index
                    )
                    .as_str(),
                );
                self.cpu.registers[dst_index as usize] = self.cpu.registers[src_index as usize];
                1
            }
            0x9000..=0x9FFF => {
                let address = instruction & 0x0FFF;
                trace!(self, "CALL [{:04X}], address: {:03X}", initial_pc, address);
                self.memory.push_to_stack_u16(&mut self.cpu.sp, self.cpu.pc);
                self.cpu.pc = address;
                4
            }
            0xA000 => {
                let ret_address = self.memory.pop_from_stack_u16(&mut self.cpu.sp);
                self.log(
                    format!(
                        "RETURN [{:04X}], returning to address: {:03X}",
                        initial_pc, ret_address
                    )
                    .as_str(),
                );
                self.cpu.pc = ret_address;
                4
            }
            0xB000..=0xB00F => {
                let keycode = (instruction & 0x000F) as u8;
                if self.input.is_pressed(keycode) {
                    self.cpu.pc = self.cpu.pc.wrapping_add(2);
                    self.log(
                        format!(
                            "WAIT_FOR_KEY [{:04X}], keycode: {:02X} is pressed, skipping next instruction",
                            initial_pc,
                            keycode
                        )
                        .as_str(),
                    );
                    3
                } else {
                    self.log(
                        format!(
                            "WAIT_FOR_KEY [{:04X}], keycode: {:02X} is not pressed, waiting",
                            initial_pc, keycode
                        )
                        .as_str(),
                    );
                    2
                }
            }
            0xC000..=0xCFFF => {
                let instr = (instruction >> 8) & 0x0F;
                let x = (instruction >> 4) & 0x0F;
                let y = instruction & 0x0F;

                let rx = self.cpu.registers[x as usize];
                let ry = self.cpu.registers[y as usize];

                let (arithmetic_function, cycles) = decode_arithmetic(instr as u8);
                let result = arithmetic_function(rx, ry);
                self.log(
                    format!(
                        "ARITHMETIC [{:04X}], reg[{}]: {:02X}, reg[{}]: {:02X}, result: {:02X}",
                        initial_pc, x, rx, y, ry, result
                    )
                    .as_str(),
                );
                self.cpu.registers[x as usize] = result;
                cycles
            }
            0xD000..=0xD00F => {
                let random_num = rand::random::<u8>();
                let reg_index = (instruction & 0x000F) as usize;
                self.log(
                    format!(
                        "RANDOM [{:04X}], random number: {:02X} stored in reg[{}]",
                        initial_pc, random_num, reg_index
                    )
                    .as_str(),
                );
                self.cpu.registers[reg_index] = random_num as u16;
                2
            }
            0xE000..=0xEFFF => {
                let address = instruction & 0x0FFF;
                self.log(
                    format!(
                        "SET_INDEX [{:04X}], index set to address: {:03X}",
                        initial_pc, address
                    )
                    .as_str(),
                );
                self.cpu.index = address;
                2
            }
            0xFFFF => {
                self.running = false;
                trace!(self, "HALT [{:04X}]", initial_pc);
                1
            }
            _ => todo!("Instruction {:04X} not implemented", instruction),
        }
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
                        let vram = &self.memory.ram[0..0x2000];
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
