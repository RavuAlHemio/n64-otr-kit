use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;
use otr_common::common::Endianness;
use otr_common::resource::ResourceHeader;
use otr_common::resource::resource_type;
use otr_common::texture::PixelFormat;
use otr_common::texture::TextureV0Header;
use otr_common::texture::TextureV1Header;


const SCALE_3_TO_8: [u8; 8] = [
    0, 36, 73, 109, 146, 182, 219, 255,
];
const SCALE_4_TO_8: [u8; 16] = [
    0, 17, 34, 51, 68, 85, 102, 119,
    136, 153, 170, 187, 204, 221, 238, 255,
];
const SCALE_5_TO_8: [u8; 32] = [
    0, 8, 16, 25, 33, 41, 49, 58,
    66, 74, 82, 90, 99, 107, 115, 123,
    132, 140, 148, 156, 165, 173, 181, 189,
    197, 206, 214, 222, 230, 239, 247, 255,
];


#[derive(Parser)]
struct Opts {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
}

fn main() {
    let opts = Opts::parse();

    // make a reader for the input
    let mut input_file = File::open(&opts.input_file)
        .expect("failed to open input file");

    // read the resource header
    let resource_header = ResourceHeader::try_read(&mut input_file)
        .expect("failed to read resource header");
    if resource_header.resource_type != resource_type::TEXTURE {
        panic!("unexpected resource type (obtained {:#010X}, expected {:#010X}", resource_header.resource_type, resource_type::TEXTURE);
    }

    // read the texture header
    let texture_header = match resource_header.version {
        0 => {
            let header = TextureV0Header::try_read(&mut input_file, resource_header.byte_order)
                .expect("failed to read v0 texture header");
            header
        },
        1 => {
            let header = TextureV1Header::try_read(&mut input_file, resource_header.byte_order)
                .expect("failed to read v1 texture header");
            header.to_v0()
        },
        other => panic!("unsupported texture version {}", other),
    };

    // read the data
    let data_length: usize = texture_header.image_data_size
        .try_into().expect("failed to convert image data size to usize");
    let mut data_buf = vec![0u8; data_length];
    input_file.read_exact(&mut data_buf)
        .expect("failed to read image data");

    // start setting up the image
    let mut png_buf = Vec::new();
    let mut png_encoder = png::Encoder::new(&mut png_buf, texture_header.width, texture_header.height);
    match texture_header.pixel_format {
        PixelFormat::R8G8B8A8 => {
            png_encoder.set_color(png::ColorType::Rgba);
            png_encoder.set_depth(png::BitDepth::Eight);
        },
        PixelFormat::R5G5B5A1 => {
            // scale manually
            png_encoder.set_color(png::ColorType::Rgba);
            png_encoder.set_depth(png::BitDepth::Eight);
        },
        // we don't have the palettes, so treat indexed as grayscale
        PixelFormat::I4|PixelFormat::G4 => {
            png_encoder.set_color(png::ColorType::Grayscale);
            png_encoder.set_depth(png::BitDepth::Four);
        },
        PixelFormat::I8|PixelFormat::G8 => {
            png_encoder.set_color(png::ColorType::Grayscale);
            png_encoder.set_depth(png::BitDepth::Eight);
        },
        PixelFormat::G3A1 => {
            // scale manually
            png_encoder.set_color(png::ColorType::GrayscaleAlpha);
            png_encoder.set_depth(png::BitDepth::Eight);
        },
        PixelFormat::G4A4 => {
            // not supported natively by PNG; scale manually
            png_encoder.set_color(png::ColorType::GrayscaleAlpha);
            png_encoder.set_depth(png::BitDepth::Eight);
        },
        PixelFormat::G8A8 => {
            png_encoder.set_color(png::ColorType::GrayscaleAlpha);
            png_encoder.set_depth(png::BitDepth::Eight);
        },
        PixelFormat::GA1 => {
            png_encoder.set_color(png::ColorType::GrayscaleAlpha);
            png_encoder.set_depth(png::BitDepth::One);
        },
        PixelFormat::Other(other) => panic!("unsupported pixel format {}", other),
    }

    let mut writer = png_encoder.write_header()
        .expect("failed to write PNG header");

    let mut image_data = Vec::new();
    match texture_header.pixel_format {
        PixelFormat::R8G8B8A8 => {
            for chunk in data_buf.chunks(4) {
                let value = resource_header.byte_order.u32_from_bytes(chunk);
                let a = u8::try_from((value >> 24) & 0xFF).unwrap();
                let b = u8::try_from((value >> 16) & 0xFF).unwrap();
                let g = u8::try_from((value >>  8) & 0xFF).unwrap();
                let r = u8::try_from((value >>  0) & 0xFF).unwrap();
                image_data.push(r);
                image_data.push(g);
                image_data.push(b);
                image_data.push(a);
            }
        },
        PixelFormat::R5G5B5A1 => {
            for chunk in data_buf.chunks(2) {
                // RRRR RGGG GGBB BBBA
                //let value = resource_header.byte_order.u16_from_bytes(chunk);
                let value = Endianness::Big.u16_from_bytes(chunk);
                let r = SCALE_5_TO_8[usize::from((value >> 11) & 0b1_1111)];
                let g = SCALE_5_TO_8[usize::from((value >> 6) & 0b1_1111)];
                let b = SCALE_5_TO_8[usize::from((value >> 1) & 0b1_1111)];
                let a = if (value & 0b1) == 0 { 0x00 } else { 0xFF };
                image_data.push(r);
                image_data.push(g);
                image_data.push(b);
                image_data.push(a);
            }
        },
        PixelFormat::I4|PixelFormat::G4|PixelFormat::I8|PixelFormat::G8|PixelFormat::G8A8 => {
            // we should be able to transfer this data raw
            image_data.extend(&data_buf);
        },
        PixelFormat::G4A4 => {
            // PNG could support this, but it doesn't; scale it manually
            for byte in &data_buf {
                let raw_intensity = ((*byte) >> 4) & 0b1111;
                let raw_alpha = ((*byte) >> 0) & 0b1111;
                let scaled_intensity = SCALE_4_TO_8[usize::from(raw_intensity)];
                let scaled_alpha = SCALE_4_TO_8[usize::from(raw_alpha)];
                image_data.push(scaled_intensity);
                image_data.push(scaled_alpha);
            }
        },
        PixelFormat::G3A1 => {
            for byte in &data_buf {
                // IIIA IIIA
                let raw_intensity1 = ((*byte) >> 5) & 0b111;
                let raw_alpha1 = ((*byte) >> 4) & 0b1;
                let raw_intensity2 = ((*byte) >> 1) & 0b111;
                let raw_alpha2 = ((*byte) >> 0) & 0b1;

                let scaled_intensity1 = SCALE_3_TO_8[usize::from(raw_intensity1)];
                let scaled_alpha1 = if raw_alpha1 == 0 { 0x0 } else { 0xFF };
                let scaled_intensity2 = SCALE_3_TO_8[usize::from(raw_intensity2)];
                let scaled_alpha2 = if raw_alpha2 == 0 { 0x0 } else { 0xFF };
                image_data.push(scaled_intensity1);
                image_data.push(scaled_alpha1);
                image_data.push(scaled_intensity2);
                image_data.push(scaled_alpha2);
            }
        },
        PixelFormat::GA1 => {
            // in this format, G = A; PNG wants those split up
            for byte in &data_buf {
                let mut oh_my_word = 0u16;
                for bit_index in (0..8).rev() {
                    let bit_is_set = (*byte & (1 << bit_index)) != 0;
                    oh_my_word <<= 2;
                    if bit_is_set {
                        oh_my_word |= 0b11;
                    }
                }
                image_data.extend(oh_my_word.to_be_bytes());
            }
        },
        PixelFormat::Other(other) => panic!("unsupported pixel format {}", other),
    }

    if let Err(e) = writer.write_image_data(&image_data) {
        panic!(
            "failed to write PNG data for {} (pixel format {:?}): {}",
            opts.input_file.display(), texture_header.pixel_format, e,
        );
    }
    writer.finish()
        .expect("failed to finish writing PNG");

    std::fs::write(&opts.output_file, &png_buf)
        .expect("failed to write out PNG file");
}
