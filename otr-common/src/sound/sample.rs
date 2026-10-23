use std::io::Read;

use crate::common::{Endianness, Error};


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SampleV0Header {
    pub loop_start: u32,
    pub loop_end: u32,
    pub loop_count: i32,
    pub loop_pad: i32,
    // state_size: u32,
    pub state: Vec<i16>, // [i16; state_size]
    pub order: i32,
    pub number_predictors: i32,
}
impl SampleV0Header {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, Error> {
        let mut fixed_buf = [0u8; 20];
        reader.read_exact(&mut fixed_buf)?;

        let loop_start = endianness.u32_from_bytes(&fixed_buf[0..4]);
        let loop_end = endianness.u32_from_bytes(&fixed_buf[4..8]);
        let loop_count = endianness.i32_from_bytes(&fixed_buf[8..12]);
        let loop_pad = endianness.i32_from_bytes(&fixed_buf[12..16]);
        let state_size_u32 = endianness.u32_from_bytes(&fixed_buf[16..20]);

        let state_size: usize = state_size_u32.try_into().unwrap();
        let mut state = Vec::with_capacity(state_size);
        for _ in 0..state_size {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            state.push(endianness.i16_from_bytes(&buf));
        }

        let mut onp_buf = [0u8; 8];
        reader.read_exact(&mut onp_buf)?;
        let order = endianness.i32_from_bytes(&onp_buf[0..4]);
        let number_predictors = endianness.i32_from_bytes(&onp_buf[4..8]);

        Ok(Self {
            loop_start,
            loop_end,
            loop_count,
            loop_pad,
            state,
            order,
            number_predictors,
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SampleV0Data {
    // book_size: u32,
    pub book: Vec<i16>, // [i16; book_size]
    // sample_size: i32,
    pub samples: Vec<u8>, // [u8; sample_size]
}
impl SampleV0Data {
    pub fn try_read<R: Read>(reader: &mut R, endianness: Endianness) -> Result<Self, Error> {
        let mut size_buf = [0u8; 4];

        reader.read_exact(&mut size_buf)?;
        let book_size_u32 = endianness.u32_from_bytes(&size_buf);
        let book_size: usize = book_size_u32.try_into().unwrap();
        let mut book = Vec::with_capacity(book_size);
        for _ in 0..book_size {
            let mut page_buf = [0u8; 2];
            reader.read_exact(&mut page_buf)?;
            book.push(endianness.i16_from_bytes(&page_buf));
        }

        reader.read_exact(&mut size_buf)?;
        let sample_size_i32 = endianness.i32_from_bytes(&size_buf);
        let sample_size: usize = sample_size_i32.try_into().unwrap();
        let mut samples = vec![0u8; sample_size];
        reader.read_exact(&mut samples)?;

        Ok(Self {
            book,
            samples,
        })
    }
}
