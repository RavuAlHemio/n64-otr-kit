use std::io::Read;

use crate::common::{Boolean, Endianness, Error};


pub mod resource_type {
    pub const DISPLAY_LIST: u32 = 0x4F_44_4C_54; // ODLT
    pub const TEXTURE: u32 = 0x4F_54_45_58; // OTEX
    pub const VERTEX: u32 = 0x4F_56_54_58; // OVTX
}


#[derive(Clone, Copy, Debug)]
pub struct ResourceHeader {
    pub byte_order: Endianness, // u8
    pub is_custom: Boolean, // u8
    pub padding0: [u8; 2],
    pub resource_type: u32,
    pub version: u32,
    pub asset_id: u64,
    pub padding1: [u8; 44],
}
impl ResourceHeader {
    pub fn try_read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0u8; 64];
        reader.read_exact(&mut buf)?;

        let byte_order = Endianness::from_base_type(buf[0]);
        if !byte_order.is_value_supported() {
            return Err(Error::UnknownEndianness(buf[0]));
        }
        let is_custom = Boolean::from_base_type(buf[1]);
        let padding0 = buf[2..4].try_into().unwrap();
        let resource_type = byte_order.u32_from_bytes(&buf[4..8]);
        let version = byte_order.u32_from_bytes(&buf[8..12]);
        let asset_id = byte_order.u64_from_bytes(&buf[12..20]);
        let padding1 = buf[20..64].try_into().unwrap();

        Ok(Self {
            byte_order,
            is_custom,
            padding0,
            resource_type,
            version,
            asset_id,
            padding1,
        })
    }
}
