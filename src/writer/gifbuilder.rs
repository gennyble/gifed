use std::convert::TryInto;

use crate::block::{extension::Extension, Block, ColorTable, ScreenDescriptor, Version};
use crate::writer::ImageBuilder;
use crate::{EncodingError, Gif};

pub struct GifBuilder {
    version: Version,
    width: u16,
    height: u16,
    background_color_index: u8,
    global_color_table: Option<ColorTable>,
    blocks: Vec<Block>,
}

impl GifBuilder {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            version: Version::Gif87a,
            width,
            height,
            background_color_index: 0,
            global_color_table: None,
            blocks: vec![],
        }
    }

    pub fn palette(mut self, palette: ColorTable) -> Self {
        self.global_color_table = Some(palette);
        self
    }

    pub fn background_index(mut self, ind: u8) -> Result<Self, EncodingError> {
        if self.global_color_table.is_none() {
            Err(EncodingError::NoColorTable)
        } else {
            self.background_color_index = ind;
            Ok(self)
        }
    }

    pub fn image(mut self, ib: ImageBuilder) -> Result<Self, EncodingError> {
        if ib.required_version() == Version::Gif89a {
            self.version = Version::Gif89a;
        }

        if let Some(gce) = ib.get_graphic_control() {
            self.blocks.push(Block::Extension(gce.into()));
        }

        self.blocks.push(Block::IndexedImage(ib.build()?));
        Ok(self)
    }

    /*pub fn extension(mut self, ext: Extension) -> Self {
        self.blocks.push(Block::Extension(ext));
        self
    }*/

    pub fn repeat(&mut self, count: u16) {
        self.blocks.push(Block::Extension(Extension::Looping(count)))
    }

    pub fn build(self) -> Gif {
        let mut lsd = ScreenDescriptor {
            width: self.width,
            height: self.height,
            packed: 0, // Set later
            background_color_index: self.background_color_index,
            pixel_aspect_ratio: 0, //TODO: Allow configuring
        };

        if let Some(gct) = &self.global_color_table {
            println!("build {}", gct.len());
            lsd.set_color_table_present(true);
            lsd.set_color_table_size((gct.len() - 1) as u8);
        }

        Gif {
            header: self.version,
            screen_descriptor: lsd,
            global_color_table: self.global_color_table,
            blocks: self.blocks,
        }
    }
}
