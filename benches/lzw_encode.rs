use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gifed::LZW;
use rand::{thread_rng, Rng};
use weezl::{encode::Encoder, BitOrder};

pub fn criterion_benchmark(c: &mut Criterion) {
	let mut random = [0u8; 2048];
	thread_rng().fill(&mut random[..]);

	c.bench_function("lzw encode 255bytes", |b| {
		b.iter(|| LZW::encode(8, black_box(&random)))
	});
	c.bench_function("weezl encode 255bytes", |b| {
		b.iter(|| {
			Encoder::new(BitOrder::Msb, 8)
				.encode(black_box(&random))
				.unwrap()
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
