use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Seek};

use crc::{Algorithm, Crc};
use zip::ZipArchive;
use zip::read::ZipFile;
use zip::result::ZipError;


// odd mix of CRC-64/XZ, /WE and /ECMA-182;
// init and xorout lead to an empty intersection
pub const CRC_64_OTR: Algorithm<u64> = Algorithm {
    width: 64,
    poly: 0x42F0E1EBA9EA3693, // common to all three above
    init: 0xFFFFFFFFFFFFFFFF, // from /XZ and /WE
    refin: false, // from /WE and /ECMA-182
    refout: false, // from /WE and /ECMA-182
    xorout: 0x0000000000000000, // from /ECMA-182
    check: 0x9D13A61C0E5B0FF5,
    residue: 0x0000000000000000,
};


#[derive(Debug)]
pub enum Error {
    Zip(ZipError),
    DuplicateCrc64 {
        one_path: String,
        other_path: String,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip(e) => write!(f, "ZIP error: {}", e),
            Self::DuplicateCrc64 { one_path, other_path }
                => write!(f, "duplicate CRC64! paths {:?} and {:?} have the same checksum", one_path, other_path),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Zip(e) => Some(e),
            Self::DuplicateCrc64 { .. } => None,
        }
    }
}
impl From<ZipError> for Error {
    fn from(value: ZipError) -> Self {
        Self::Zip(value)
    }
}


#[derive(Clone, Debug)]
pub struct Otr2Archive<R: Read + Seek> {
    zip_archive: ZipArchive<R>,
    crc64_to_file_path: BTreeMap<u64, String>,
}
impl<R: Read + Seek> Otr2Archive<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        let zip_archive = ZipArchive::new(reader)?;
        let mut crc64_to_file_path = BTreeMap::new();
        let crc = Crc::<u64>::new(&CRC_64_OTR);
        for entry_res in zip_archive.file_names() {
            let entry = entry_res?;
            let csum = crc.checksum(entry.as_bytes());
            let dupe_opt = crc64_to_file_path.insert(
                csum,
                entry.clone().into_owned(),
            );
            if let Some(dupe) = dupe_opt {
                return Err(Error::DuplicateCrc64 {
                    one_path: dupe,
                    other_path: entry.into_owned(),
                });
            }
        }
        Ok(Self {
            zip_archive,
            crc64_to_file_path,
        })
    }

    pub fn entry_by_crc64(&mut self, crc: u64) -> Result<ZipFile<'_, R>, ZipError> {
        let path = self.crc64_to_file_path.get(&crc)
            .ok_or(ZipError::FileNotFound)?;
        self.zip_archive.by_name(path)
    }

    pub fn crc64_to_file_path(&self) -> &BTreeMap<u64, String> {
        &self.crc64_to_file_path
    }
}
impl<R: Read + Seek> AsRef<ZipArchive<R>> for Otr2Archive<R> {
    fn as_ref(&self) -> &ZipArchive<R> {
        &self.zip_archive
    }
}
impl<R: Read + Seek> AsMut<ZipArchive<R>> for Otr2Archive<R> {
    fn as_mut(&mut self) -> &mut ZipArchive<R> {
        &mut self.zip_archive
    }
}
