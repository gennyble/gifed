use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::{
    block::{
        extension::{Application, Extension, GraphicControl},
        Block, ColorTable, CompressedImage, ImageDescriptor, IndexedImage, ScreenDescriptor,
        Version,
    },
    color, Gif,
};

pub struct GifReader {}

impl GifReader {
    pub fn file<P: AsRef<Path>>(path: P) -> Gif {
        let mut file = File::open(path).expect("Failed to open file");
        let mut reader = SmartReader {
            inner: vec![],
            position: 0,
        };
        file.read_to_end(&mut reader.inner)
            .expect("Failed to read gif");

        let mut gif = Self::read_required(&mut reader);

        if gif.screen_descriptor.color_table_present() {
            let gct_size = gif.screen_descriptor.color_table_len() * 3;
            gif.global_color_table = Some(Self::read_color_table(&mut reader, gct_size));
        }

        loop {
            match Self::read_block(&mut reader) {
                Some(block) => {
                    /*match &block {
                        Block::IndexedImage(_) => println!("Indexed Image"),
                        Block::BlockedImage(_) => println!("Blocked Image"),
                        Block::Extension(ext) => match ext {
                            Extension::GraphicControl(_) => println!("Graphic Cotrol Extension"),
                            Extension::Looping(_) => println!("Netscape Extension"),
                            Extension::Comment(vec) => {
                                println!("Comment Extension {:X}", vec.len())
                            }
                            Extension::Application(_) => todo!(),
                        },
                    }*/

                    gif.blocks.push(block)
                }
                None => return gif,
            }
        }
    }

    fn read_required(reader: &mut SmartReader) -> Gif {
        let version = match reader.take_lossy_utf8(6).as_deref() {
            Some("GIF87a") => Version::Gif87a,
            Some("GIF89a") => Version::Gif89a,
            _ => panic!("Version string is unknown"),
        };

        let mut lsd_buffer: [u8; 7] = [0; 7];
        reader
            .read_exact(&mut lsd_buffer)
            .expect("Failed to read Logical Screen Descriptor from gif");

        let lsd = ScreenDescriptor::from(lsd_buffer);

        Gif {
            header: version,
            screen_descriptor: lsd,
            global_color_table: None,
            blocks: vec![],
        }
    }

    fn read_color_table(reader: &mut SmartReader, size: usize) -> ColorTable {
        let buffer = reader
            .take(size as usize)
            .expect("Failed to read Color Table");

        ColorTable::try_from(&buffer[..]).expect("Failed to parse Color Table")
    }

    fn read_block(reader: &mut SmartReader) -> Option<Block> {
        let block_id = reader.u8().expect("File ended early");

        match block_id {
            0x21 => Some(Self::read_extension(reader)),
            0x2C => Some(Self::read_image(reader)),
            0x3B => None,
            _ => None, /*panic!(
                           "Unknown block identifier {:X} {:X}",
                           block_id, reader.position
                       ),*/
        }
    }

    fn read_extension(reader: &mut SmartReader) -> Block {
        let extension_id = reader.u8().expect("File ended early");

        match extension_id {
            0xF9 => {
                reader.skip(1); // Skip block length, we know it
                let mut data = [0u8; 4];
                reader
                    .read_exact(&mut data)
                    .expect("Data ended early in graphics control extension sublock");
                reader.skip(1); // Skip block terminator

                Block::Extension(Extension::GraphicControl(GraphicControl::from(data)))
            }
            0xFE => Block::Extension(Extension::Comment(reader.take_and_collapse_subblocks())),
            0x01 => todo!(), // plain text extension
            0xFF => {
                assert_eq!(Some(11), reader.u8());
                let identifier = reader.take_lossy_utf8(8).unwrap().to_string();
                let authentication_code: [u8; 3] =
                    TryInto::try_into(reader.take(3).unwrap()).unwrap();
                let data = reader.take_and_collapse_subblocks();

                Block::Extension(Extension::Application(Application {
                    identifier,
                    authentication_code,
                    data,
                }))
            }
            _ => panic!("Unknown Extension Identifier!"),
        }
    }

    fn read_image(mut reader: &mut SmartReader) -> Block {
        let mut buffer = [0u8; 9];
        reader
            .read_exact(&mut buffer)
            .expect("Failed to read Image Descriptor");
        let descriptor = ImageDescriptor::from(buffer);

        let color_table = if descriptor.color_table_present() {
            let size = descriptor.color_table_size() * 3;
            Some(Self::read_color_table(&mut reader, size))
        } else {
            None
        };

        let lzw_csize = reader.u8().expect("Failed to read LZW Minimum Code Size");

        let compressed_data = reader.take_and_collapse_subblocks();
        println!("c{}", compressed_data.len());

        let mut decompress = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, lzw_csize);
        //TODO: remove unwrap
        let mut decompressed_data = decompress.decode(&compressed_data).unwrap();

        Block::IndexedImage(IndexedImage {
            image_descriptor: descriptor,
            local_color_table: color_table,
            indicies: decompressed_data,
        })
    }
}

struct SmartReader {
    inner: Vec<u8>,
    position: usize,
}

impl SmartReader {
    pub fn u8(&mut self) -> Option<u8> {
        self.position += 1;
        self.inner.get(self.position - 1).map(|b| *b)
    }

    pub fn u16(&mut self) -> Option<u16> {
        self.position += 2;
        self.inner
            .get(self.position - 2..self.position)
            .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn skip(&mut self, size: usize) {
        self.position += size;
    }

    pub fn take(&mut self, size: usize) -> Option<&[u8]> {
        self.position += size;
        self.inner.get(self.position - size..self.position)
    }

    //TODO: Result not Option when buffer len
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Option<()> {
        if self.position + buf.len() > self.inner.len() {
            None
        } else {
            self.position += buf.len();
            buf.copy_from_slice(&self.inner[self.position - buf.len()..self.position]);
            Some(())
        }
    }

    pub fn take_vec(&mut self, size: usize) -> Option<Vec<u8>> {
        self.position += size;
        self.inner
            .get(self.position - size..self.position)
            .map(|bytes| bytes.to_vec())
    }

    pub fn take_lossy_utf8(&mut self, size: usize) -> Option<Cow<'_, str>> {
        self.take(size).map(|bytes| String::from_utf8_lossy(bytes))
    }

    pub fn take_data_subblocks(&mut self) -> Vec<Vec<u8>> {
        let mut blocks = vec![];

        loop {
            let block_size = self.u8().expect("Failed to read length of sublock");

            if block_size == 0 {
                return blocks;
            }

            let block = self
                .take_vec(block_size as usize)
                .expect("Failed to read sublock");

            blocks.push(block);
        }
    }

    pub fn take_and_collapse_subblocks(&mut self) -> Vec<u8> {
        let blocks = self.take_data_subblocks();
        let mut ret = vec![];
        for block in blocks {
            ret.extend_from_slice(&block)
        }

        ret
    }
}
