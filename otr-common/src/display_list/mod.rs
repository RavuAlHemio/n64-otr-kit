pub mod color_source;
pub mod f3d;
pub mod f3dex;
pub mod f3dex2;
pub(crate) mod shared;


use std::fmt;
use std::io::Read;

use from_to_repr::from_to_other;

use crate::common::Endianness;
use crate::display_list::color_source::ColorSource;


#[derive(Debug)]
pub enum Operand {
    /// The pointer to the vertex data.
    V,

    /// The offset to the vertex data within the vertex data table of the asset.
    ///
    /// These are alternate semantics for the V operand within some OTR-specific operations.
    VOtr,

    /// The index of the first slot of the vertex buffer to consider.
    V0,

    /// The index beyond the last slot of the vertex buffer to consider.
    VLast,

    /// The number of vectors to consider.
    N,

    /// A vertex index of a triangle.
    TriangleVertexIndex { triangle_index: u8, vertex_index: u8 },
}
impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V => write!(f, "v"),
            Self::VOtr => write!(f, "v (OTR)"),
            Self::V0 => write!(f, "v0"),
            Self::VLast => write!(f, "v_last"),
            Self::N => write!(f, "n"),
            Self::TriangleVertexIndex { triangle_index, vertex_index }
                => write!(f, "triangle{}_vertex_indexes[{}]", triangle_index, vertex_index),
        }
    }
}


#[derive(Debug)]
pub enum Error {
    /// A size field must correlate with other fields in the operation, but doesn't.
    ///
    /// `calculated` is the expected size calculated from the other fields; `stored` is the value
    /// stored in the operation itself.
    SizeMismatch { calculated: usize, stored: usize },

    /// An operand must be, but isn't, a multiple of a number.
    OperandNotMultiple { operand: Operand, multiple_of: u64, value: u64 },

    /// Subtracting two operands underflowed.
    SubtractionUnderflow { minuend: Operand, subtrahend: Operand },

    /// The source of operations stopped providing values midway through an operation.
    SourceEndedEarly,

    /// Encountered an operation whose opcode is not known to us.
    UnsupportedOpcode { opcode: u8 },

    /// The display list is encoded in a Graphics Binary Interface unsupported by us.
    UnsupportedGbiVersion { version: GbiVersion },
}
impl Error {
    pub fn ensure_operand_multiple(operand: Operand, multiple_of: u64, value: u64) -> Result<(), Self> {
        if value % multiple_of == 0 {
            Ok(())
        } else {
            Err(Self::OperandNotMultiple { operand, multiple_of, value })
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SizeMismatch { calculated, stored }
                => write!(f, "size mismatch: calculated {}, stored {}", calculated, stored),
            Self::OperandNotMultiple { operand, multiple_of, value }
                => write!(f, "operand {} must be a multiple of {} but is {}", operand, multiple_of, value),
            Self::SubtractionUnderflow { minuend, subtrahend }
                => write!(f, "subtracting {} from {} underflowed", subtrahend, minuend),
            Self::SourceEndedEarly
                => write!(f, "operations stream ended early"),
            Self::UnsupportedOpcode { opcode }
                => write!(f, "unknown opcode {:#04X}", opcode),
            Self::UnsupportedGbiVersion { version }
                => write!(f, "unknown GBI version {:?}", version),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SizeMismatch { .. } => None,
            Self::OperandNotMultiple { .. } => None,
            Self::SubtractionUnderflow { .. } => None,
            Self::SourceEndedEarly => None,
            Self::UnsupportedOpcode { .. } => None,
            Self::UnsupportedGbiVersion { .. } => None,
        }
    }
}


#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum GbiVersion {
    F3db = 0,
    F3d = 1,
    F3dEx = 2,
    F3dExB = 3,
    F3dEx2 = 4,
    Other(u8),
}

#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum BranchFlag {
    /// Call the target.
    ///
    /// Once the target finishes executing, execution continues with the instruction after the call
    /// instruction.
    Call = 0,

    /// Jump to the target.
    ///
    /// Once the target finishes executing, execution stops.
    Jump = 1,

    Other(u8),
}

#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum ColorFormat {
    Rgba = 0,
    Yuv = 1,
    Palette = 2, // aka ColorIndexed (CI)
    GrayscaleAlpha = 3, // aka IntensityAlpha (IA)
    Grayscale = 4, // aka Intensity (I)
    Other(u8),
}

#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum BitsPerPixel {
    B4 = 0,
    B8 = 1,
    B16 = 2,
    B32 = 3,
    Other(u8),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GraphicsCommand {
    /// Load `n` vertices from memory at `v` into the vertex buffer at offset `v0`.
    Vertex {
        v: u32,
        n: u8,
        v0: u8,
    },

    /// Load a matrix from memory onto the matrix stack.
    Matrix {
        /// The memory location from which to load the matrix.
        matrix: u32,

        /// Whether to push the matrix on top of the matrix stack instead of replacing the topmost
        /// element on the stack.
        ///
        /// If `true`, pushes the matrix up onto the matrix stack. If `false`, replaces the topmost
        /// element on the matrix stack.
        push: bool,

        /// Whether to store the matrix directly instead of multiplying it with the topmost element
        /// on the stack.
        ///
        /// If `true`, stores the matrix. If `false`, stores the product of this matrix (left
        /// operand) multiplied by the topmost element on the matrix stack (right operand).
        load: bool,

        /// Whether to affect the projection matrix instead of the model view matrix.
        ///
        /// If `true`, the projection matrix is affected. If `false`, the model view matrix is
        /// affected.
        projection: bool,
    },

    /// Draw a triangle whose vertices are found in the vertex buffer at the given indexes.
    Triangle1 {
        triangle_vertex_indexes: [u8; 3],
        flag: Option<u8>,
    },

    /// Draw two triangles whose vertices are found in the vertex buffer at the given indexes.
    ///
    /// The draw operation is the same as used for [`Triangle1`](Self::Triangle1) except executed
    /// twice; the flag is assumed to be 0.
    Triangle2 {
        triangle0_vertex_indexes: [u8; 3],
        triangle1_vertex_indexes: [u8; 3],
    },

    /// Set and/or clear bits in the current geometry mode field.
    ///
    /// The resulting geometry mode becomes `((geometry_mode & (!clear_bits)) | set_bits)`.
    ///
    /// This is a merged opcode introduced in F3DEX2; the previous individual opcodes can be
    /// translated into this one as follows:
    ///
    /// * SetGeometryMode(bits) = GeometryMode(bits, 0)
    /// * ClearGeometryMode(bits) = GeometryMode(0, bits)
    /// * LoadGeometryMode(bits) = GeometryMode(bits, 0xFFFFFF)
    GeometryMode {
        set_bits: u32,
        clear_bits: u32,
    },

    /// Set the texturing parameters.
    TexturingParameters {
        s_scale: u16,
        t_scale: u16,
        max_mipmap_levels: u8,
        tile_number: u8,
        on: bool,
    },

    /// Set the color for drawing primitives.
    SetPrimitiveColor {
        lod_minimum_value_clamp: u8,
        lod_fraction: u8,
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    },

    /// Set settings for color combination.
    SetColorCombine {
        cycle_0_color_a: ColorSource,
        cycle_0_color_b: ColorSource,
        cycle_0_color_c: ColorSource,
        cycle_0_color_d: ColorSource,
        cycle_0_alpha_a: ColorSource,
        cycle_0_alpha_b: ColorSource,
        cycle_0_alpha_c: ColorSource,
        cycle_0_alpha_d: ColorSource,
        cycle_1_color_a: ColorSource,
        cycle_1_color_b: ColorSource,
        cycle_1_color_c: ColorSource,
        cycle_1_color_d: ColorSource,
        cycle_1_alpha_a: ColorSource,
        cycle_1_alpha_b: ColorSource,
        cycle_1_alpha_c: ColorSource,
        cycle_1_alpha_d: ColorSource,
    },

    /// Set the color with which to fill.
    SetFillColor {
        color: u32,
    },

    /// Set the environment color value.
    ///
    /// The environment color is one of the possible inputs to the color combiner; see
    /// [`SetColorCombine`](Self::SetColorCombine).
    SetEnvironmentColor {
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    },

    /// Modifies bits in the "other mode" word of the RDP.
    ///
    /// Depending on `upper_half`, operates on the upper or the lower half of the "other mode" word.
    /// Takes `length` bits, the lowest of which is specified with `shift`, and replaces them with
    /// the corresponding (preshifted) bits in `data`.
    ///
    /// The operation is analogous to:
    /// ```rust,ignore
    /// let reg = if upper_half { &mut other_mode_upper } else { &mut other_mode_lower };
    /// *reg = (*reg & (!(((1 << length) - 1) << shift))) | data;
    /// ```
    SetOtherMode {
        upper_half: bool,
        shift: u8,
        length: u8,
        data: u32,
    },

    /// Set up multi-tile texture modes.
    SetTile {
        color_format: ColorFormat,
        bits_per_pixel: BitsPerPixel,
        values_64bit_per_row: u16,
        tmem_texture_offset: u16,
        tile_descriptor_index: u8,
        palette_index: u8,
        clamp_t: bool,
        mirror_t: bool,
        mask_t: u8,
        shift_t: u8,
        clamp_s: bool,
        mirror_s: bool,
        mask_s: u8,
        shift_s: u8,
    },

    /// Select the part of the texture which to tile.
    SetTileSize {
        tile_descriptor_index: u8,
        upper_left_s: u16,
        upper_left_t: u16,
        lower_right_s: u16,
        lower_right_t: u16,
    },

    /// Set the image that will be used for texture loading to the image at the given memory
    /// address.
    SetTextureImage {
        color_format: ColorFormat,
        bits_per_pixel: BitsPerPixel,
        width: u16,
        image_address: u32,
    },

    /// Loads a block of data into texture memory.
    ///
    /// The source data has previously been specified by [`SetTextureImage`](Self::SetTextureImage)
    /// or a similar operation.
    LoadBlock {
        tile_number: u8,
        upper_left_s: u16,
        upper_left_t: u16,
        lower_right_s: u16,
        t_change_per_scan_line: u16,
    },

    /// Branch to another display list in memory.
    BranchDisplayList {
        display_list: u32,
        branch_flag: BranchFlag,
    },

    /// Synchronize the attribute interpolation pipe.
    ///
    /// Changes to rendering attributes are not necessarily immediate; if an attribute change is
    /// followed by draw commands, the new attributes might only take effect midway through drawing.
    /// This command ensures that processing the display list pauses until the attributes have been
    /// internalized by the renderer.
    RdpPipeSync,

    /// Synchronize the texture loading pipe.
    ///
    /// Texture loads are not necessarily immediate. This command pauses drawing to ensure that the
    /// next draw command uses the newly loaded texture.
    RdpLoadSync,

    /// Marks the end of a display list.
    EndDisplayList,

    /// Load `n` vertices from offset `v` in the asset identified by `hash` into the vertex buffer
    /// at offset `v0`.
    ///
    /// This operation is an OTR extension.
    OtrVertexByHash {
        v: u32,
        n: u8,
        v0: u8,
        hash: u64,
    },

    /// Branch to the display list in the asset identified by `hash`.
    ///
    /// This operation is an OTR extension.
    OtrBranchDisplayListByHash {
        branch_flag: BranchFlag,
        hash: u64,
    },

    /// Set the image that will be used for texture loading to the image in the asset with the given
    /// hash.
    ///
    /// This operation is an OTR extension.
    OtrSetTextureImageByHash {
        color_format: ColorFormat,
        bits_per_pixel: BitsPerPixel,
        width: u16,
        hash: u64,
    },

    /// Load a matrix from the asset identified by `hash` onto the matrix stack.
    ///
    /// The other operands are the same as in the [`Matrix`](Self::Matrix) variant.
    ///
    /// This operation is an OTR extension.
    OtrMatrixByHash {
        hash: u64,
        push: bool,
        load: bool,
        projection: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DisplayListV0Header {
    pub gbi_version: GbiVersion, // u8
    pub padding0: [u8; 7],
    pub marker: u32,
    pub marker_data: u32,
    pub resource_hash: u64, // stored as [u32; 2]
}
impl DisplayListV0Header {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, crate::common::Error> {
        let mut buf = [0u8; 24];
        reader.read_exact(&mut buf)?;

        let gbi_version = GbiVersion::from_base_type(buf[0]);
        let padding0 = buf[1..8].try_into().unwrap();
        let marker = endianness.u32_from_bytes(&buf[8..12]);
        let marker_data = endianness.u32_from_bytes(&buf[12..16]);
        let hash_hi = endianness.u32_from_bytes(&buf[16..20]);
        let hash_lo = endianness.u32_from_bytes(&buf[20..24]);
        let resource_hash =
            (u64::from(hash_hi) << 32)
            | (u64::from(hash_lo) << 0)
        ;

        Ok(Self {
            gbi_version,
            padding0,
            marker,
            marker_data,
            resource_hash,
        })
    }

    pub fn try_read_data<R: Read>(&self, reader: &mut R, endianness: Endianness) -> Result<Vec<GraphicsCommand>, crate::common::Error> {
        // read all the data we can
        let mut words = Vec::new();
        loop {
            let mut buf = [0u8; 4];
            let actually_read = reader.read(&mut buf)?;
            if actually_read == 0 {
                // end of the line
                break;
            }
            if actually_read < buf.len() {
                // read the rest
                reader.read_exact(&mut buf[actually_read..])?;
            }
            let word = endianness.u32_from_bytes(&buf);
            words.push(word);
        }

        // parse it out
        let mut word_iterator = words.iter().copied();
        let mut commands = Vec::with_capacity(words.len() / 2);
        match self.gbi_version {
            GbiVersion::F3d => {
                while let Some(command) = crate::display_list::f3d::take_next_command(&mut word_iterator)? {
                    commands.push(command);
                }
            },
            GbiVersion::F3dEx => {
                while let Some(command) = crate::display_list::f3dex::take_next_command(&mut word_iterator)? {
                    commands.push(command);
                }
            },
            GbiVersion::F3dEx2 => {
                while let Some(command) = crate::display_list::f3dex2::take_next_command(&mut word_iterator)? {
                    commands.push(command);
                }
            },
            other => return Err(Error::UnsupportedGbiVersion { version: other }.into()),
        }

        Ok(commands)
    }
}
