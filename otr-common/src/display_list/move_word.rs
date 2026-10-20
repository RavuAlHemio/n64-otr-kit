use from_to_repr::from_to_other;

use crate::display_list::GbiVersion;


#[allow(unused)]
mod g {
    pub const MW_MATRIX: u8 = 0;
    pub const MW_NUMLIGHT: u8 = 2;
    pub const MW_CLIP: u8 = 4;
    pub const MW_SEGMENT: u8 = 6;
    pub const MW_FOG: u8 = 8;
    pub const MW_GENSTAT: u8 = 8;
    pub const MW_LIGHTCOL: u8 = 10;
    pub const MW_POINTS: u8 = 12; // F3D, F3DEX
    pub const MW_FORCEMTX: u8 = 12; // F3DEX2
    pub const MW_PERSPNORM: u8 = 14;

    pub const MWO_NUMLIGHT: u16 = 0x00;
    pub const MWO_CLIP_RNX: u16 = 0x04;
    pub const MWO_CLIP_RNY: u16 = 0x0C;
    pub const MWO_CLIP_RPX: u16 = 0x14;
    pub const MWO_CLIP_RPY: u16 = 0x1C;
    // MWO_SEGMENT_[0-9A-F] are just 4 * index
    pub const MWO_FOG: u16 = 0;
    pub const MWO_A_LIGHT_1: u16 = 0;
    pub const MWO_B_LIGHT_1: u16 = 4;
    pub const MWO_MATRIX_XX_XY_I: u16 = 0x00;
    pub const MWO_MATRIX_XZ_XW_I: u16 = 0x04;
    pub const MWO_MATRIX_YX_YY_I: u16 = 0x08;
    pub const MWO_MATRIX_YZ_YW_I: u16 = 0x0C;
    pub const MWO_MATRIX_ZX_ZY_I: u16 = 0x10;
    pub const MWO_MATRIX_ZZ_ZW_I: u16 = 0x14;
    pub const MWO_MATRIX_WX_WY_I: u16 = 0x18;
    pub const MWO_MATRIX_WZ_WW_I: u16 = 0x1C;
    pub const MWO_MATRIX_XX_XY_F: u16 = 0x20;
    pub const MWO_MATRIX_XZ_XW_F: u16 = 0x24;
    pub const MWO_MATRIX_YX_YY_F: u16 = 0x28;
    pub const MWO_MATRIX_YZ_YW_F: u16 = 0x2C;
    pub const MWO_MATRIX_ZX_ZY_F: u16 = 0x30;
    pub const MWO_MATRIX_ZZ_ZW_F: u16 = 0x34;
    pub const MWO_MATRIX_WX_WY_F: u16 = 0x38;
    pub const MWO_MATRIX_WZ_WW_F: u16 = 0x3C;
    pub const MWO_POINT_RGBA: u16 = 0x10;
    pub const MWO_POINT_ST: u16 = 0x14;
    pub const MWO_POINT_XYSCREEN: u16 = 0x18;
    pub const MWO_POINT_ZSCREEN: u16 = 0x1C;
}


#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u16, derive_compare = "as_int")]
pub enum ClipMoveWordOffset {
    NegativeX = 0x04,
    NegativeY = 0x0C,
    PositiveX = 0x14,
    PositiveY = 0x1C,
    Other(u16),
}

#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u16, derive_compare = "as_int")]
pub enum LightColorMoveWordOffset {
    ALight1 = 0x00,
    BLight1 = 0x04,
    ALight2 = 0x18,
    BLight2 = 0x1C,
    ALight3 = 0x30,
    BLight3 = 0x34,
    ALight4 = 0x48,
    BLight4 = 0x4C,
    ALight5 = 0x60,
    BLight5 = 0x64,
    ALight6 = 0x78,
    BLight6 = 0x7C,
    ALight7 = 0x90,
    BLight7 = 0x94,
    ALight8 = 0xA8,
    BLight8 = 0xAC,
    Other(u16),
}

#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum PointsMoveWordProperty {
    ColorRgba = 0x10,
    TextureCoordinatesSt = 0x14,
    ScreenCoordinatesXy = 0x18,
    ScreenCoordinateZ = 0x1C,
    Other(u8),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MoveWordTarget {
    Matrix { offset: u16 },
    NumberOfLights,
    Clip { offset: ClipMoveWordOffset },
    Segment { segment_index: u16 }, // segment_index = offset * 4
    Fog,
    StatusId { status_id: u16 },
    LightColor { offset: LightColorMoveWordOffset },
    Points { vertex_index: u16, property: PointsMoveWordProperty }, // offset = vertex_index * 40 + property
    ForceMatrix,
    PerspectiveNormalization,
    Other { index: u8, offset: u16 },
}
impl MoveWordTarget {
    pub fn index(&self) -> u8 {
        match self {
            Self::Matrix { .. } => 0,
            Self::NumberOfLights => 2,
            Self::Clip { .. } => 4,
            Self::Segment { .. } => 6,
            Self::Fog => 8,
            Self::StatusId { .. } => 8,
            Self::LightColor { .. } => 10,
            Self::Points { .. } => 12, // F3D/F3DEX
            Self::ForceMatrix => 12, // F3DEX2
            Self::PerspectiveNormalization => 14,
            Self::Other { index, .. } => *index,
        }
    }

    pub fn from_index_offset_data(gbi_version: GbiVersion, index: u8, offset: u16, data: u32) -> Self {
        if index == g::MW_FOG {
            return if offset == g::MWO_FOG {
                Self::Fog
            } else {
                Self::StatusId { status_id: offset }
            };
        }
        if gbi_version != GbiVersion::F3dBeta && gbi_version != GbiVersion::F3dExBeta {
            if index == g::MW_PERSPNORM && offset == 0 {
                return Self::PerspectiveNormalization;
            }
        }
        if index == g::MW_SEGMENT && offset % 4 == 0 {
            let segment_index = offset / 4;
            return Self::Segment { segment_index };
        }
        if index == g::MW_NUMLIGHT && offset == g::MWO_NUMLIGHT {
            return Self::NumberOfLights;
        }
        if gbi_version.is_f3d() || gbi_version == GbiVersion::F3dExBeta {
            // (other GBIs have their own operation, G_MODIFYVTX)
            if index == g::MW_POINTS {
                let vertex_index = offset / 40;
                let property_u8 = u8::try_from(offset % 40).unwrap();
                let property = PointsMoveWordProperty::from_base_type(property_u8);
                return Self::Points { vertex_index, property };
            }
        }
        if gbi_version == GbiVersion::F3dEx2 {
            if index == g::MW_FORCEMTX && offset == 0 && data == 0x10000 {
                return Self::ForceMatrix;
            }
        }
        if index == g::MW_MATRIX {
            return Self::Matrix { offset };
        }
        if index == g::MW_CLIP {
            return Self::Clip {
                offset: ClipMoveWordOffset::from_base_type(offset),
            };
        }
        if index == g::MW_LIGHTCOL {
            return Self::LightColor {
                offset: LightColorMoveWordOffset::from_base_type(offset),
            };
        }
        Self::Other { index, offset }
    }
}
