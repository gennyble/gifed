pub struct ColorImage {
	width: u16,
	height: u16,
	data: Vec<Pixel>
}

impl ColorImage {
	pub fn new() {
		
	}
}

pub enum Pixel {
	Color(Color),
	Transparent
}