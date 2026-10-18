pub mod f3d;
pub mod f3dex;
pub mod f3dex2;
pub(crate) mod shared;


use std::fmt;
use std::io::Read;

use from_to_repr::from_to_other;

use crate::common::Endianness;


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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GraphicsCommand {
    /// Load `n` vertices from memory at `v` into the vertex buffer at offset `v0`.
    Vertex {
        v: u32,
        n: u8,
        v0: u8,
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

    /// Synchronizes the attribute interpolation pipe.
    ///
    /// Changes to rendering attributes are not necessarily immediate; if an attribute change is
    /// followed by draw commands, the new attributes might only take effect midway through drawing.
    /// This command ensures that processing the display list pauses until the attributes have been
    /// internalized by the renderer.
    RdpPipeSync,

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
