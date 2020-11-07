use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use gifed::LZW;

pub fn criterion_benchmark(c: &mut Criterion) {
	let mut random = [0u8; 255];
	thread_rng().fill(&mut random[..]);

    c.bench_function("lzw encode 255bytes", |b| b.iter(|| LZW::encode(8, black_box(&random))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);