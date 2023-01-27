mod mp3;

fn main() {
	let file = std::env::args().nth(1).unwrap();
	let data = std::fs::read(file).unwrap();
	let mut breaker = mp3::Breaker::new();
	breaker.split(data).unwrap();
}
