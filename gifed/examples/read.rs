use gifed::{
	reader::{self, GifReader},
	writer::ImageBuilder,
	Gif,
};

fn main() {
	let reader = GifReader::file("examples/simulation.gif").unwrap();
	let first = reader.images().next().unwrap();

	Gif::builder(first.width, first.height)
		.palette(first.palette.clone())
		.image(
			ImageBuilder::new(first.width, first.height)
				.transparent_index(first.transparent_index)
				.indicies(first.indicies),
		)
		.build()
		.unwrap()
		.save("first.gif")
		.unwrap();
}
