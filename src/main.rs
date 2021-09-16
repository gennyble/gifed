use std::{fs::File, io::Write};

use gifed::{
    block::{
        extension::{Extension, GraphicControl},
        BlockedImage, ColorTable, Version,
    },
    reader::GifReader,
    writer::{GifBuilder, ImageBuilder},
    Color,
};
use owo_colors::OwoColorize;

fn main() {
    write_test();
    //return;
    let gif = GifReader::file("test.gif");
    //let gif = GifReader::file("/home/gen/Downloads/tas_tillie.gif");

    println!("Version {}", gif.header.yellow());
    println!(
        "Logical Screen Descriptor\n\tDimensions {}x{}",
        gif.screen_descriptor.width.yellow(),
        gif.screen_descriptor.height.yellow()
    );

    if gif.screen_descriptor.color_table_present() {
        println!(
            "\tGlobal Color Table Present {}\n\tGlobal Color Table Size {}",
            "Yes".green(),
            gif.screen_descriptor.color_table_len().green()
        );
    } else {
        println!(
            "\tGlobal Color Table Present {}\n\tGlobal Color Table Size {}",
            "No".red(),
            gif.screen_descriptor.color_table_len().red()
        );
    }

    for block in gif.blocks {
        match block {
            gifed::block::Block::IndexedImage(_) => todo!(),
            gifed::block::Block::BlockedImage(bli) => describe_blocked_image(bli),
            gifed::block::Block::Extension(ext) => match ext {
                gifed::block::extension::Extension::GraphicControl(gce) => {
                    println!(
                        "Graphic Control Extension\n\tDelay Time {}",
                        format!("{}s", gce.delay_time() as f32 / 100.0).yellow()
                    )
                }
                gifed::block::extension::Extension::Looping(_) => todo!(),
                gifed::block::extension::Extension::Comment(cmt) => {
                    println!("Comment Extension\n\tLength {}", cmt.len())
                }
                gifed::block::extension::Extension::Application(app) => {
                    let auth = app.authentication_code();
                    println!("Application Extension\n\tIdentifier {}\n\tAuthentication {:02X} {:02X} {:02X}",app.identifier().yellow(), auth[0].yellow(), auth[1].yellow(), auth[2].yellow());
                }
            },
        }
    }
}

fn describe_blocked_image(bli: BlockedImage) {
    println!(
        "Image\n\tOffset {}x{}\n\tDimensions {}x{}",
        bli.image_descriptor.left.yellow(),
        bli.image_descriptor.top.yellow(),
        bli.image_descriptor.width.yellow(),
        bli.image_descriptor.height.yellow(),
    );

    if bli.image_descriptor.color_table_present() {
        println!(
            "\tLocal Color Table Present {}\n\tLocal Color Table Size {}",
            "Yes".green(),
            bli.image_descriptor.color_table_size().green()
        );
    } else {
        println!(
            "\tLocal Color Table Present {}\n\tLocal Color Table Size {}",
            "No".red(),
            bli.image_descriptor.color_table_size().red()
        );
    }

    println!(
        "\t{} image data sub-blocks totaling {} bytes",
        bli.blocks.len().yellow(),
        bli.blocks
            .iter()
            .map(|vec| vec.len())
            .sum::<usize>()
            .yellow()
    )
}

fn write_test() {
    const size: u16 = 256;
    let gcon = GraphicControl::new(
        gifed::block::extension::DisposalMethod::Clear,
        false,
        false,
        25,
        0,
    );

    let mut gct = ColorTable::new();
    gct.push(Color {
        r: 0x44,
        g: 0x44,
        b: 0x44,
    });
    gct.push(Color {
        r: 0x88,
        g: 0x44,
        b: 0xDD,
    });

    println!("{} {}", gct.packed_len(), gct.len());

    let mut builder = GifBuilder::new(Version::Gif89a, size, size).global_color_table(gct);

    for x in 4..16 {
        let mut raw = vec![0; size as usize * size as usize];

        for i in x * 7..x * 7 + 32 {
            for j in x * 7..x * 7 + 32 {
                let index = i * size as usize + j;
                raw[index as usize] = 1;
            }
        }

        builder = builder
            .extension(Extension::GraphicControl(gcon.clone()))
            .image(ImageBuilder::new(size, size).indicies(raw));
    }

    builder = builder.extension(Extension::Looping(0));

    let vec = builder.build().to_vec();
    File::create("test.gif").unwrap().write_all(&vec).unwrap();

    let mut text_gif = File::create("test_gif.txt").unwrap();
    for byte in vec {
        text_gif
            .write_all(&format!("{:02X} ", byte).as_bytes())
            .unwrap();
    }
}

fn write_static_test() {
    const size: u16 = 256;

    let mut gct = ColorTable::new();
    gct.push(Color {
        r: 0x44,
        g: 0x44,
        b: 0x44,
    });
    gct.push(Color {
        r: 0x55,
        g: 0x44,
        b: 0x88,
    });

    println!("{} {}", gct.packed_len(), gct.len());

    let mut builder = GifBuilder::new(Version::Gif87a, size, size).global_color_table(gct);

    let x = 4;
    let mut raw = vec![0; size as usize * size as usize];

    for i in x * 7..x * 7 + 8 {
        for j in x * 7..x * 7 + 8 {
            let index = i * size as usize + j;
            raw[index as usize] = 1;
        }
    }

    builder = builder.image(ImageBuilder::new(size, size).indicies(raw));

    //builder = builder.extension(Extension::Looping(0));

    let vec = builder.build().to_vec();
    File::create("test.gif").unwrap().write_all(&vec).unwrap();

    let mut text_gif = File::create("test_gif.txt").unwrap();
    for byte in vec {
        text_gif
            .write_all(&format!("{:02X} ", byte).as_bytes())
            .unwrap();
    }
}
