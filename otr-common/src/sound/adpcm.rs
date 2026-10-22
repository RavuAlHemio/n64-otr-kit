use std::io::{self, Write};

use crate::common::Endianness;


pub fn adpcm_to_pcm<W: Write>(
    samples: &[u8],
    codebook: &[[[i16; 8]; 2]],
    output: &mut W,
    output_endianness: Endianness,
) -> Result<(), io::Error> {
    assert_eq!(samples.len() % 9, 0);
    assert_eq!(codebook.len(), 8);

    let mut lookback = [0i32; 2];
    for sample_chunk in samples.chunks(9) {
        let header = sample_chunk[0];
        let in_samples = &sample_chunk[1..9];
        let mut out_samples = [0i16; 16];

        decode_adpcm_frame(
            header,
            in_samples,
            &mut out_samples,
            codebook,
            &mut lookback,
        );
        for sample in out_samples {
            let bs = output_endianness.i16_to_bytes(sample);
            output.write_all(&bs)?;
        }
    }

    Ok(())
}

fn decode_adpcm_frame(
    header: u8,
    in_samples: &[u8],
    out_samples: &mut [i16],
    codebook: &[[[i16; 8]; 2]],
    lookback: &mut [i32],
) {
    assert_eq!(in_samples.len(), 8);
    assert_eq!(out_samples.len(), 16);
    assert_eq!(codebook.len(), 8);
    assert_eq!(lookback.len(), 2);

    let shift = header >> 4;
    let table_index = usize::from(header & 0xF);
    let table = codebook[table_index];

    let mut in_sample_index = 0;
    let mut out_sample_index = 0;
    for _ in 0..2 {
        let mut ins = [0i16; 8];
        for j in 0..4 {
            ins[j*2] = sign_extend_i4_to_i16(in_samples[in_sample_index] >> 4) << shift;
            ins[j*2 + 1] = sign_extend_i4_to_i16(in_samples[in_sample_index] & 0xF) << shift;
            in_sample_index += 1;
        }
        for j in 0..8 {
            let mut acc = 0;
            acc += i32::from(table[0][j]) * lookback[1];
            acc += i32::from(table[1][j]) * lookback[0];
            acc += i32::from(ins[j]) << 11;
            for k in 0..j {
                acc += i32::from(table[1][(j - k) - 1]) * i32::from(ins[k]);
            }
            acc >>= 11;
            let final_sample = if acc > i32::from(i16::MAX) {
                i16::MAX
            } else if acc < i32::from(i16::MIN) {
                i16::MIN
            } else {
                acc as i16
            };
            out_samples[out_sample_index] = final_sample;
            out_sample_index += 1;
        }

        // remember final samples for lookback
        lookback[1] = i32::from(out_samples[out_sample_index - 2]);
        lookback[0] = i32::from(out_samples[out_sample_index - 1]);
    }
}

const fn sign_extend_i4_to_i16(value: u8) -> i16 {
    if (value & 0b1000) != 0 {
        ((value as u16) | 0b1111_1111_1111_1000) as i16
    } else {
        value as i16
    }
}
