use std::io::{Read, Write};

use crate::common::{Endianness, Error};


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SequenceV0Header {
    pub id: u32,
    // bank_count: u32,
    pub banks: Vec<String>, // [String; bank_count]
    // with String { length: u32, bytes: [u8; length] }
    pub data_byte_count: u32,
}
impl SequenceV0Header {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, Error> {
        let mut fixed_buf = [0u8; 8];
        reader.read_exact(&mut fixed_buf)?;

        let id = endianness.u32_from_bytes(&fixed_buf[0..4]);
        let bank_count_u32 = endianness.u32_from_bytes(&fixed_buf[4..8]);
        let bank_count: usize = bank_count_u32.try_into().unwrap();

        let mut banks = Vec::with_capacity(bank_count);
        for _ in 0..bank_count {
            let mut length_buf = [0u8; 4];
            reader.read_exact(&mut length_buf)?;
            let length_u32 = endianness.u32_from_bytes(&length_buf);
            let length: usize = length_u32.try_into().unwrap();

            let mut bank_buf = vec![0u8; length];
            reader.read_exact(&mut bank_buf)?;

            banks.push(String::from_utf8(bank_buf).unwrap());
        }

        let mut count_buf = [0u8; 4];
        reader.read_exact(&mut count_buf)?;
        let data_byte_count = endianness.u32_from_bytes(&count_buf);

        Ok(Self {
            id,
            banks,
            data_byte_count,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W, endianness: Endianness) -> Result<(), Error> {
        endianness.write_u32(self.id, writer)?;

        let bank_count_u32: u32 = self.banks.len().try_into().unwrap();
        endianness.write_u32(bank_count_u32, writer)?;
        for bank_name in &self.banks {
            let bank_name_len_u32: u32 = bank_name.len()
                .try_into().unwrap();
            endianness.write_u32(bank_name_len_u32, writer)?;
            writer.write_all(bank_name.as_bytes())?;
        }

        let data_byte_count_u32: u32 = self.data_byte_count.try_into().unwrap();
        endianness.write_u32(data_byte_count_u32, writer)?;
        Ok(())
    }
}
