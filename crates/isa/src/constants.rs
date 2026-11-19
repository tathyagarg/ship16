pub const ENTRY_POINT: u16 = 0x1000;
pub const ROM_SIZE: usize = 0x4000; // 48KB
pub const RAM_BASE: usize = ROM_SIZE;
pub const RAM_SIZE: usize = 0xC000; // 16KB
pub const STACK_TOP: u16 = 0xFFFF;

pub const SCREEN_HEIGHT: usize = 64;
pub const SCREEN_WIDTH: usize = 64;
pub const VRAM_SIZE: usize = SCREEN_HEIGHT * SCREEN_WIDTH;
pub const STACK_LIMIT: usize = RAM_BASE + VRAM_SIZE;

pub const CYCLES_PER_FRAME: u64 = 16667; // 60 FPS
pub const INTERRUPT_ENTRY_CYCLES: u32 = 4;
