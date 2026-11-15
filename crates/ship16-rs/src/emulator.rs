use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crate::{
    cpu,
    eval::Evaluate,
    input,
    memory::{self, RoomTooLarge},
    ppu, timer,
};
use isa::{
    constants::*,
    instruction::{Instr, decode},
    interrupt::{Interrupt, InterruptController},
};

macro_rules! trace {
    ($emu:expr, $($arg:tt)*) => {
        if let Some(w) = $emu.logger.as_mut() {
            let _ = writeln!(w, $($arg)*);
        }
    };
}

pub struct Rng(u32);

impl Rng {
    fn new() -> Self {
        Rng(0xDEADBEEF)
    }

    fn next_u16(&mut self) -> u16 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x & 0xFFFF) as u16
    }
}

pub struct Emulator {
    cpu: cpu::Cpu,
    memory: memory::Memory,
    timer: timer::Timer,
    ppu: ppu::Ppu,
    input: input::Input,
    logger: Option<BufWriter<File>>,
    rng: Rng,

    interrupt_controller: InterruptController,
    running: bool,

    frame_ready: bool,
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
            rng: Rng::new(),
            interrupt_controller: InterruptController::new(),
            running: false,

            frame_ready: false,
        }
    }

    pub fn vram(&self) -> &[u8] {
        self.memory.vram()
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), RoomTooLarge> {
        self.memory.load_rom(data)
    }

    pub fn log(&mut self, message: &str) {
        if let Some(logger) = &mut self.logger {
            use std::io::Write;
            let _ = writeln!(logger, "{}", message);
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn tick(&mut self, cycles: u32) {
        if self.timer.tick_n(cycles as u16) {
            self.interrupt_controller
                .request_interrupt(Interrupt::Timer);
        }

        if self.ppu.tick_n(cycles as u64) {
            self.interrupt_controller
                .request_interrupt(Interrupt::VBlank);
            self.frame_ready = true;
        }
    }

    pub fn push(&mut self, value: u16, pc: u16) -> bool {
        match self.cpu.push_u16(&mut self.memory, value) {
            Ok(_) => true,
            Err(e) => {
                self.log(&format!("Stack overflow at {:04X}: {:?}", pc, e));
                self.running = false;
                false
            }
        }
    }

    pub fn pop(&mut self, pc: u16) -> Option<u16> {
        match self.cpu.pop_u16(&self.memory) {
            Ok(value) => Some(value),
            Err(e) => {
                self.log(&format!("Stack underflow at {:04X}: {:?}", pc, e));
                self.running = false;
                None
            }
        }
    }

    pub fn start(&mut self) {
        self.running = true;
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

    pub fn key_down(&mut self, key: u8) {
        if self.input.key_down(key) {
            self.interrupt_controller
                .request_interrupt(Interrupt::Input);
        }
    }

    pub fn key_up(&mut self, key: u8) {
        self.input.key_up(key);
    }

    pub fn execute_instruction(&mut self) -> u32 {
        let pc = self.cpu.pc;
        let word = self.memory.read_u16(pc);
        let ext = self.memory.read_u16(pc.wrapping_add(2));

        self.cpu.pc = self.cpu.pc.wrapping_add(2);

        let instr = match decode(word, ext) {
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
            Instr::Halt => self.running = false,
            Instr::Wait => self.cpu.waiting = true,

            Instr::Load8(reg_x) => {
                let value = self.memory.read_byte(self.cpu.index);
                self.cpu.registers[reg_x] = value as u16;
            }
            Instr::Load16(reg_x) => {
                let value = self.memory.read_u16(self.cpu.index);
                self.cpu.registers[reg_x] = value;
            }
            Instr::Store8(reg_x) => {
                let value = self.cpu.registers[reg_x] as u8;
                self.memory.write_byte(self.cpu.index, value);
            }
            Instr::Store16(reg_x) => {
                let value = self.cpu.registers[reg_x];
                self.memory.write_byte(self.cpu.index, (value & 0xFF) as u8);
                self.memory
                    .write_byte(self.cpu.index.wrapping_add(1), (value >> 8) as u8);
            }
            Instr::AddIndex(reg_x) => {
                let value = self.cpu.registers[reg_x];
                self.cpu.index = self.cpu.index.wrapping_add(value);
            }

            Instr::Push(reg_x) => {
                let value = self.cpu.registers[reg_x];
                if let Err(e) = self.cpu.push_u16(&mut self.memory, value) {
                    self.log(&format!("Stack overflow at {:04X}: {:?}", pc, e));
                    self.running = false;
                }
            }
            Instr::Pop(reg_x) => match self.cpu.pop_u16(&self.memory) {
                Ok(value) => self.cpu.registers[reg_x] = value,
                Err(e) => {
                    self.log(&format!("Stack underflow at {:04X}: {:?}", pc, e));
                    self.running = false;
                }
            },
            Instr::SkipIfFlag { flag, set } => {
                let flag_set = self.cpu.flags.get(flag);
                skip = flag_set == set;
            }

            Instr::TimerCount(reg_x) => self.timer.set_counter(self.cpu.registers[reg_x]),
            Instr::TimerReload(reg_x) => self.timer.set_reload(self.cpu.registers[reg_x]),
            Instr::TimerRead(reg_x) => self.cpu.registers[reg_x] = self.timer.counter,
            Instr::TimerConfig { enable, periodic } => {
                self.timer.enabled = enable;
                self.timer.periodic = periodic;
            }

            Instr::IntEnable(mask) => self.interrupt_controller.enable(mask),
            Instr::IntDisable(mask) => self.interrupt_controller.disable(mask),
            Instr::SetVector(interrupt) => self
                .interrupt_controller
                .set_vector(interrupt, self.cpu.index),
            Instr::Ei => self.interrupt_controller.set_master(true),
            Instr::Di => self.interrupt_controller.set_master(false),
            Instr::Reti => {
                if let Some(address) = self.pop(pc) {
                    self.cpu.pc = address;
                    self.interrupt_controller.set_master(true);
                }
            }

            Instr::Clear => self.ppu.clear(self.memory.vram_mut()),
            Instr::DrawColor(color) => self.ppu.draw_pixel(self.memory.vram_mut(), color),
            Instr::DrawColorReg(r) => {
                let color = self.cpu.registers[r as usize] as u8;
                self.ppu.draw_pixel(self.memory.vram_mut(), color);
            }
            Instr::DrawSprite(index) => self.draw_sprite(index),
            Instr::DrawSpriteReg(r) => self.draw_sprite(self.cpu.registers[r as usize] as u8),
            Instr::MoveTo(packed) => self.ppu.set_cursor_packed(packed),
            Instr::AddX(v) => self.ppu.move_x(v as i16),
            Instr::SubX(v) => self.ppu.move_x(-(v as i16)),
            Instr::AddY(v) => self.ppu.move_y(v as i16),
            Instr::SubY(v) => self.ppu.move_y(-(v as i16)),
            Instr::SetXReg(r) => self.ppu.set_x(self.cpu.registers[r as usize]),
            Instr::SetYReg(r) => self.ppu.set_y(self.cpu.registers[r as usize]),

            Instr::Jump(address) | Instr::JumpLong(address) => self.cpu.pc = address,
            Instr::Call(address) | Instr::CallLong(address) => {
                let ret = self.cpu.pc;
                if self.push(ret, pc) {
                    self.cpu.pc = address;
                }
            }
            Instr::Return => {
                if let Some(address) = self.pop(pc) {
                    self.cpu.pc = address;
                }
            }
            Instr::SkipIf { cond, reg_x, reg_y } => {
                skip = cond.eval(self.cpu.registers[reg_x], self.cpu.registers[reg_y]);
            }
            Instr::SkipIfKey(key) => skip = self.input.is_pressed(key),

            Instr::SetReg { reg, value } => self.cpu.registers[reg as usize] = value as u16,
            Instr::SetRegLong { reg, value } => self.cpu.registers[reg as usize] = value,
            Instr::Copy {
                reg_x: src,
                reg_y: dst,
            } => {
                self.cpu.registers[dst as usize] = self.cpu.registers[src as usize];
            }
            Instr::Alu { op, reg_x, reg_y } => {
                let out = op.eval(self.cpu.registers[reg_x], self.cpu.registers[reg_y]);
                self.cpu.registers[reg_x] = out.result;
                self.cpu.flags.set_from_alu(&out);
            }
            Instr::Random(r) => self.cpu.registers[r as usize] = self.rng.next_u16() as u16,
            Instr::SetIndex(address) | Instr::SetIndexLong(address) => self.cpu.index = address,
        }

        if skip {
            let next = self.memory.read_u16(self.cpu.pc);
            let word_count = match next {
                0xF000 | 0xF100 | 0xF200 | 0xF300..=0xF30F => 2,
                _ => 1,
            };

            self.cpu.pc = self.cpu.pc.wrapping_add(word_count * 2);
        }

        instr.base_cycles() + skip as u32
    }

    pub fn run_frame(&mut self) {
        self.frame_ready = false;
        while self.running && !self.frame_ready {
            self.step();
        }
    }

    pub fn step(&mut self) -> u32 {
        if !self.running {
            return 0;
        }

        let cycles = self.step_cpu();
        self.tick(cycles);
        cycles
    }

    pub fn step_cpu(&mut self) -> u32 {
        if self.cpu.waiting {
            if self.interrupt_controller.has_pending() {
                self.cpu.waiting = false;
            } else {
                return 1;
            }
        }

        if let Some(cycles) = self.service_interrupt() {
            return cycles;
        }

        self.execute_instruction()
    }

    pub fn service_interrupt(&mut self) -> Option<u32> {
        let (_, vector) = self.interrupt_controller.acknowledge()?;
        let ret = self.cpu.pc;

        if !self.push(ret, self.cpu.pc) {
            return None;
        }
        self.cpu.pc = vector;

        Some(INTERRUPT_ENTRY_CYCLES)
    }
}
