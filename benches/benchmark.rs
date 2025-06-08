use armagnac::{
    core::ArmVersion::V7M, decoder::Lut16AndGrouped32InstructionDecoder, harness::ElfHarness,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

pub fn benchmark(c: &mut Criterion) {
    let elf = include_bytes!("benchmark.elf");
    let mut harness = ElfHarness::new(elf);

    let mut g = c.benchmark_group("group");
    g.sample_size(20);
    g.measurement_time(Duration::from_secs(60));

    // Test with basic instruction decoder
    g.bench_function("math_default_decoder", |b| {
        b.iter(|| black_box(harness.call1("bench_math", 5.0f32.to_bits())))
    });

    // Test with faster instruction decoder
    harness.proc.instruction_decoder = Box::new(Lut16AndGrouped32InstructionDecoder::new(V7M));

    g.bench_function("math_lut16grouped32_decoder", |b| {
        b.iter(|| black_box(harness.call1("bench_math", 5.0f32.to_bits())))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
