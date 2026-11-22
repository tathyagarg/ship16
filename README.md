<div align="center">
  <img src=".github/assets/logo.svg" alt="SHIP-16 Logo" width="600"/>
  <h1>Ship-16</h1>

<div>
  <img alt="GitHub commit activity" src="https://img.shields.io/github/commit-activity/w/tathyagarg/ship16?style=for-the-badge">  
  <img alt="GitHub License" src="https://img.shields.io/github/license/tathyagarg/ship16?style=for-the-badge"/>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white"/>
</div>
<p>
  SHIP-16 is an experimental 16-bit virtual computer implemented in Rust. It includes a custom instruction set, an assembler, and an emulator with memory, graphics, and input systems. Its design and specifications are documented in the accompanying technical specification.
  </p>
</div>

> [!NOTE]
> For the complete technical specification, see **[the specification](./spec.pdf)**.

## Getting Started

Clone the repository:

```bash
git clone https://github.com/tathyagarg/ship-16
cd ship-16
```

Build the project:

```bash
cargo build --release
```

Run an example program:

```bash
cargo run -p ship16-rs --release -- --rom roms/hello_world.rom
```

## About

SHIP-16 is designed as a simple virtual computer intended to be a step-up from the CHIP-8 architecture. It features a 16-bit architecture, a custom instruction set, simple graphics system, and keyboard input. The example emulator is implemented in Rust.

The source consists of three crates:
- `ship16-rs`: The emulator implementation in Rust.
- `isa`: The instruction set architecture definition.
- `asm`: The assembler implementation.

## Demos

| Demo | Preview | Note |
| :--- | :--- | :--- |
| Hello World | <img src=".github/assets/hello_world.gif" width="300" alt="Hello World Demo"/> | |
| Bounce | <img src=".github/assets/bounce.gif" width="300" alt="Bounce Demo"/> | |
| BSOD | <img src=".github/assets/demo.gif" width="300" alt="BSOD Demo"/> | The screen turns blue when any (recognized) key is pressed. |
| Sprite Demo | <img src=".github/assets/sprite.gif" width="300" alt="Sprite Demo"/> | The smiley (both neutral and smiling) is a custom sprite encoded in the ROM. It can be controlled with keyboard input. When smiling, the sprite moves faster. |
| Conway's Game of Life (Gosper Glider Gun) | <img src=".github/assets/gol.gif" width="300" alt="Conway's Game of Life Demo"/> | A simple implementation of Conway's Game of Life (Made by AI) Not shown in the video is the pre-simulation mode where you can set up the starting state of the board with the help of a controllable cursor (1: left, 2: right, 0: up, 9: down, F: toggle dead/alive, A: start simulation). After starting the simulation, step through the iterations by pressing B or toggle the automatic mode by pressing E. |

## License

SHIP-16 is licensed under the [WTFPL](http://www.wtfpl.net/). Do whatever you want with it.
