<div align="center">
  <img src=".github/assets/logo.svg" alt="SHIP-16 Logo" width="300"/>
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

For the complete technical specification, see **[the specification](./spec.pdf)**.

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
cargo run --bin asm -- --input crates/asm/in/bsod.s --output output.rom &> /dev/null
cargo run --bin ship16 -- --rom output.rom
```

### Expected Output

When you run the example program, you should see rapidly changing random colors on the screen, that turn into a completely blue screen when you press any key (0-9, a-f).

## Demos

| Demo | Preview |
| :--- | :--- |
| Hello World | <img src=".github/assets/hello_world.gif" width="300" alt="Hello World Demo"/> |
| Bounce | <img src=".github/assets/bounce.gif" width="300" alt="Bounce Demo"/> |
| BSOD | <img src=".github/assets/demo.gif" width="300" alt="BSOD Demo"/> |

## License

SHIP-16 is licensed under the [WTFPL](http://www.wtfpl.net/). Do whatever you want with it.
