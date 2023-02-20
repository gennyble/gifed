use std::fs::File;

use camino::{Utf8Path, Utf8PathBuf};
use colorsquash::{ColorCollector, Squasher};
use gifed::{
	block::{extension::DisposalMethod, Palette},
	writer::{ImageBuilder, Writer},
	Color,
};

fn main() {
	let base = Utf8PathBuf::from("/Users/gen/microglass-transfer/transcube");
	let mut collector = ColorCollector::new();

	let mut imgs = vec![];
	for idx in 1..=120 {
		let img = load_and_rgb(base.join(format!("{idx:04}.png")));
		collector.add(&img.b);
		imgs.push(img);
	}

	let squash: Squasher<u8> = collector.as_squasher(255);
	let pal = squash.palette();
	let trans_idx = pal
		.iter()
		.enumerate()
		.find(|(_idx, rgb)| rgb[0] == 0 && rgb[1] == 0 && rgb[2] == 0)
		.unwrap()
		.0;

	let mut palette = Palette::new();
	for c in pal {
		palette.push(Color::new(c[0], c[1], c[2]));
	}
	println!("{}", palette.len());

	let f = File::create(base.join("transcube.gif")).unwrap();
	let mut write = Writer::new(f, imgs[0].w as u16, imgs[1].h as u16, Some(palette)).unwrap();

	for img in imgs {
		let mut buf = vec![0; img.w as usize * img.h as usize];
		squash.map_unsafe(&img.b, &mut buf);
		let img = ImageBuilder::new(img.w as u16, img.h as u16)
			.transparent_index(Some(trans_idx as u8))
			.delay(3)
			.disposal_method(DisposalMethod::RestoreBackground)
			.build(buf)
			.unwrap();

		write.image(img).unwrap();
	}

	write.repeat(gifed::block::LoopCount::Forever).unwrap();
	write.done().unwrap();
}

pub fn load_and_rgb<P: AsRef<Utf8Path>>(path: P) -> Img {
	let decoder = png::Decoder::new(File::open(path.as_ref()).unwrap());
	let mut reader = decoder.read_info().unwrap();
	let mut b = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut b).unwrap();

	let b = (&b[..info.buffer_size()]).to_owned();

	Img {
		b: rgba_replace_a(b, (0, 0, 0), 128),
		w: info.width,
		h: info.height,
	}
}

pub fn rgba_replace_a(buf: Vec<u8>, repl: (u8, u8, u8), tol: u8) -> Vec<u8> {
	let mut r = vec![0; (buf.len() / 4) * 3];

	for (idx, px) in buf.chunks(4).enumerate() {
		if px[3] >= tol {
			r[idx * 3] = px[0];
			r[idx * 3 + 1] = px[1];
			r[idx * 3 + 2] = px[2];
		} else {
			r[idx * 3] = repl.0;
			r[idx * 3 + 1] = repl.1;
			r[idx * 3 + 2] = repl.2;
		}
	}

	r
}

pub struct Img {
	b: Vec<u8>,
	w: u32,
	h: u32,
}
