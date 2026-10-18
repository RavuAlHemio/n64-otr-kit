use std::fmt;
use std::io;

use from_to_repr::from_to_other;


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    UnknownEndianness(u8),
    DisplayList(crate::display_list::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e)
                => write!(f, "I/O error: {}", e),
            Self::UnknownEndianness(value)
                => write!(f, "unknown endianness value: {}", value),
            Self::DisplayList(e)
                => write!(f, "display list error: {}", e),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::UnknownEndianness(_) => None,
            Self::DisplayList(e) => Some(e),
        }
    }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<crate::display_list::Error> for Error {
    fn from(value: crate::display_list::Error) -> Self {
        Self::DisplayList(value)
    }
}


macro_rules! impl_num_from_bytes {
    ($name:ident, $type:ty) => {
        pub fn $name(&self, bytes: &[u8]) -> $type {
            use std::mem::size_of;
            if bytes.len() != size_of::<$type>() {
                panic!("expected {} bytes, obtained {} ({:?})", size_of::<$type>(), bytes.len(), bytes);
            }
            let bytes_array: [u8; size_of::<$type>()] = bytes.try_into().unwrap();
            match self {
                Self::Little => <$type>::from_le_bytes(bytes_array),
                Self::Big => <$type>::from_be_bytes(bytes_array),
                other => panic!("unsupported endian type {:?}", other),
            }
        }
    };
}


#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum Endianness {
    Little = 0,
    Big = 1,
    Other(u8),
}
impl Endianness {
    #[cfg(target_endian = "little")]
    pub const fn native() -> Self {
        Self::Little
    }

    #[cfg(target_endian = "big")]
    pub const fn native() -> Self {
        Self::Big
    }

    pub const fn is_value_supported(&self) -> bool {
        match self {
            Self::Other(_) => false,
            _ => true,
        }
    }

    impl_num_from_bytes!(i16_from_bytes, i16);
    impl_num_from_bytes!(u16_from_bytes, u16);
    impl_num_from_bytes!(u32_from_bytes, u32);
    impl_num_from_bytes!(u64_from_bytes, u64);
    impl_num_from_bytes!(f32_from_bytes, f32);
}

#[derive(Clone, Copy, Debug)]
#[from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum Boolean {
    False = 0,
    True = 1,
    Other(u8),
}
impl Boolean {
    pub const fn as_bool(&self) -> bool {
        match self {
            Self::False => false,
            Self::True => true,
            Self::Other(o) => *o != 0,
        }
    }
}
impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}
impl From<Boolean> for bool {
    fn from(value: Boolean) -> Self {
        value.as_bool()
    }
}
