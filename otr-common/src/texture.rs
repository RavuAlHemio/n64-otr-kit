use std::io::Read;

use from_to_repr::from_to_other;
use ordered_float::OrderedFloat;

use crate::common::{Endianness, Error};


#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u32, derive_compare = "as_int")]
pub enum PixelFormat {
    R8G8B8A8 = 1,
    R5G5B5A1 = 2,
    I4 = 3,
    I8 = 4,
    G4 = 5,
    G8 = 6,
    G3A1 = 7,
    G4A4 = 8,
    G8A8 = 9,
    GA1 = 10,
    Other(u32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextureV0Header {
    pub pixel_format: PixelFormat, // u32
    pub width: u32,
    pub height: u32,
    pub image_data_size: u32,
}
impl TextureV0Header {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, Error> {
        let mut buf = [0u8; 16];
        reader.read_exact(&mut buf)?;

        let pixel_format = PixelFormat::from_base_type(endianness.u32_from_bytes(&buf[0..4]));
        let width = endianness.u32_from_bytes(&buf[4..8]);
        let height = endianness.u32_from_bytes(&buf[8..12]);
        let image_data_size = endianness.u32_from_bytes(&buf[12..16]);

        Ok(Self {
            pixel_format,
            width,
            height,
            image_data_size,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextureV1Header {
    pub pixel_format: PixelFormat, // u32
    pub width: u32,
    pub height: u32,
    pub flags: u32,
    pub horizontal_byte_scale: OrderedFloat<f32>,
    pub vertical_pixel_scale: OrderedFloat<f32>,
    pub image_data_size: u32,
}
impl TextureV1Header {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, Error> {
        let mut buf = [0u8; 28];
        reader.read_exact(&mut buf)?;

        let pixel_format = PixelFormat::from_base_type(endianness.u32_from_bytes(&buf[0..4]));
        let width = endianness.u32_from_bytes(&buf[4..8]);
        let height = endianness.u32_from_bytes(&buf[8..12]);
        let flags = endianness.u32_from_bytes(&buf[12..16]);
        let horizontal_byte_scale = OrderedFloat(endianness.f32_from_bytes(&buf[16..20]));
        let vertical_pixel_scale = OrderedFloat(endianness.f32_from_bytes(&buf[20..24]));
        let image_data_size = endianness.u32_from_bytes(&buf[24..28]);

        Ok(Self {
            pixel_format,
            width,
            height,
            flags,
            horizontal_byte_scale,
            vertical_pixel_scale,
            image_data_size,
        })
    }

    pub fn to_v0(&self) -> TextureV0Header {
        TextureV0Header {
            pixel_format: self.pixel_format,
            width: self.width,
            height: self.height,
            image_data_size: self.image_data_size,
        }
    }
}
