use criterion::{Criterion, black_box, criterion_group, criterion_main};

use ship16_rs::Emulator;

fn benchmark_cpu(c: &mut Criterion) {
    let rom = std::fs::read("../../roms/benchmark.rom").unwrap();

    c.bench_function("cpu_execution", |b| {
        b.iter_batched(
            || {
                let mut emu = Emulator::new();
                emu.start();
                emu.load_rom(&rom).unwrap();
                emu
            },
            |mut emu| {
                for _ in 0..1000 {
                    emu.run_frame();
                }
                black_box(emu)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, benchmark_cpu);
criterion_main!(benches);
