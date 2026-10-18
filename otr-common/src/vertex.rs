use std::io::Read;

use crate::common::{Endianness, Error};


#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VertexV0Header {
    pub vertex_count: u32,
}
impl VertexV0Header {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;

        let vertex_count = endianness.u32_from_bytes(&buf[0..4]);

        Ok(Self {
            vertex_count,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VertexV0 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub flag: u16,
    pub texture_s: i16,
    pub texture_t: i16,
    pub red_or_normal_x: u8,
    pub green_or_normal_y: u8,
    pub blue_or_normal_z: u8,
    pub alpha: u8,
}
impl VertexV0 {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, Error> {
        let mut buf = [0u8; 16];
        reader.read_exact(&mut buf)?;

        let x = endianness.i16_from_bytes(&buf[0..2]);
        let y = endianness.i16_from_bytes(&buf[2..4]);
        let z = endianness.i16_from_bytes(&buf[4..6]);
        let flag = endianness.u16_from_bytes(&buf[6..8]);
        let texture_s = endianness.i16_from_bytes(&buf[8..10]);
        let texture_t = endianness.i16_from_bytes(&buf[10..12]);
        let red_or_normal_x = buf[12];
        let green_or_normal_y = buf[13];
        let blue_or_normal_z = buf[14];
        let alpha = buf[15];

        Ok(Self {
            x,
            y,
            z,
            flag,
            texture_s,
            texture_t,
            red_or_normal_x,
            green_or_normal_y,
            blue_or_normal_z,
            alpha,
        })
    }
}
