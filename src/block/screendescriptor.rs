use std::convert::TryInto;

pub struct ScreenDescriptor {
    pub width: u16,
    pub height: u16,
    pub packed: u8,
    pub background_color_index: u8,
    pub pixel_aspect_ratio: u8,
}

impl ScreenDescriptor {
    pub fn set_color_table_present(&mut self, is_present: bool) {
        if is_present {
            self.packed |= 0b1000_0000;
        } else {
            self.packed &= 0b0111_1111;
        }
    }

    pub fn set_color_table_size(&mut self, size: u8) {
        println!("scts: {}", size);
        // GCT size is calulated by raising two to this number plus one,
        // so we have to work backwards.
        let size = (size as f32).log2().ceil() - 1f32;
        self.packed |= size as u8;
    }

    //TODO: Setter for sort flag in packed field
    //TODO: Setter for color resolution in packed field

    pub fn color_table_present(&self) -> bool {
        self.packed & 0b1000_0000 != 0
    }

    pub fn color_table_len(&self) -> usize {
        crate::packed_to_color_table_length(self.packed & 0b0000_0111)
    }
}

impl From<&ScreenDescriptor> for Box<[u8]> {
    fn from(lsd: &ScreenDescriptor) -> Self {
        let mut vec = vec![];
        vec.extend_from_slice(&lsd.width.to_le_bytes());
        vec.extend_from_slice(&lsd.height.to_le_bytes());
        vec.push(lsd.packed);
        vec.push(lsd.background_color_index);
        vec.push(lsd.pixel_aspect_ratio);

        vec.into_boxed_slice()
    }
}

impl From<[u8; 7]> for ScreenDescriptor {
    fn from(arr: [u8; 7]) -> Self {
        let width = u16::from_le_bytes(arr[0..2].try_into().unwrap());
        let height = u16::from_le_bytes(arr[2..4].try_into().unwrap());
        let packed = arr[4];
        let background_color_index = arr[5];
        let pixel_aspect_ratio = arr[6];

        Self {
            width,
            height,
            packed,
            background_color_index,
            pixel_aspect_ratio,
        }
    }
}
