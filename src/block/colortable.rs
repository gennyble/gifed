pub use crate::Color;
use std::{
    convert::{TryFrom, TryInto},
    ops::Deref,
};

pub struct ColorTable {
    table: Vec<Color>,
}

impl ColorTable {
    pub fn new() -> Self {
        Self { table: vec![] }
    }

    /// Returns the number of colors in the color table as used by the packed
    /// fields in the Logical Screen Descriptor and Image Descriptor. You can
    /// get the actual size with the [`len`](struct.ColorTable.html#method.len) method.
    pub fn packed_len(&self) -> u8 {
        ((self.table.len() as f32).log2().ceil() - 1f32) as u8
    }

    /// Returns the number of items in the table
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Pushes a color on to the end of the table
    pub fn push(&mut self, color: Color) {
        self.table.push(color);
    }
}

impl Deref for ColorTable {
    type Target = [Color];

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl From<Vec<Color>> for ColorTable {
    fn from(table: Vec<Color>) -> Self {
        ColorTable { table }
    }
}

impl From<&ColorTable> for Box<[u8]> {
    fn from(table: &ColorTable) -> Self {
        let mut vec = vec![];

        for color in table.iter() {
            vec.extend_from_slice(&[color.r, color.g, color.b]);
        }

        let packed_len = 2usize.pow(table.packed_len() as u32 + 1);
        let padding = (packed_len as usize - table.len()) * 3;
        if padding > 0 {
            vec.extend_from_slice(&vec![0; padding]);
        }

        vec.into_boxed_slice()
    }
}

//TODO: TryFrom Vec<u8> (must be multiple of 3 len) and From Vec<Color>
impl TryFrom<&[u8]> for ColorTable {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() % 3 != 0 {
            return Err(());
        } else {
            Ok(Self {
                table: value
                    .chunks(3)
                    .map(|slice| Color::from(TryInto::<[u8; 3]>::try_into(slice).unwrap()))
                    .collect::<Vec<Color>>(),
            })
        }
    }
}
