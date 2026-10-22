use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use clap::Parser;
use otr_common::common::Endianness;
use otr_common::resource::ResourceHeader;
use otr_common::resource::resource_type;
use otr_common::sound::adpcm::adpcm_to_pcm;
use otr_common::sound::sample::{SampleV0Data, SampleV0Header};


#[derive(Parser)]
struct Opts {
    pub input_aufc: PathBuf,
    pub output_wav: PathBuf,
}


fn main() {
    let opts = Opts::parse();

    let mut aufc = File::open(&opts.input_aufc)
        .expect("failed to open input file");
    let resource_header = ResourceHeader::try_read(&mut aufc)
        .expect("failed to read resource header");
    if resource_header.resource_type != resource_type::SAMPLE {
        panic!("wrong input resource type");
    }
    if resource_header.version != 0 {
        panic!("can only deal with v0 samples");
    }

    let _header = SampleV0Header::try_read(&mut aufc, resource_header.byte_order)
        .expect("failed to read sample header");
    let data = SampleV0Data::try_read(&mut aufc, resource_header.byte_order)
        .expect("failed to read sample data");

    if data.book.len() % (2*8) != 0 {
        panic!("wtf, ADPCM codebook has {} entries; expected a multiple of {}", data.book.len(), 2*8);
    }

    let mut source_index = 0;
    let mut codebook = [[[0i16; 8]; 2]; 8];
    for page in &mut codebook {
        for side in page {
            for word in side {
                if source_index < data.book.len() {
                    *word = data.book[source_index];
                    source_index += 1;
                }
            }
        }
    }

    let mut output_bs = Vec::new();
    let mut output_cursor = Cursor::new(&mut output_bs);

    adpcm_to_pcm(
        &data.samples,
        &codebook,
        &mut output_cursor,
        Endianness::Little,
    )
        .expect("failed to encode ADPCM to PCM");

    // write it out as .wav; assume sampling rate of 22050

    // (technically, RIFF is a container format,
    // but since it doesn't store anything *after* the data,
    // we can pretend it's a header+data format)
    const WAV_HEADER_SIZE: u32 =
        4 + 4 + 4 // RIFF chunk header
        + 4 + 4 + 2 + 2 + 4 + 4 + 2 + 2 // "fmt " chunk size
        + 4 + 4 // "data" chunk header size
    ;
    let sample_data_len: u32 = output_bs.len().try_into().unwrap();
    let riff_size = ((WAV_HEADER_SIZE - 8) + sample_data_len).to_le_bytes();

    const SAMPLE_RATE: u32 = 22_050;
    const NUM_CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;

    let bytes_per_block = NUM_CHANNELS * (BITS_PER_SAMPLE / 8);
    let bytes_per_sec = SAMPLE_RATE * u32::from(bytes_per_block);

    let num_channels_bs = NUM_CHANNELS.to_le_bytes();
    let sample_rate_bs = SAMPLE_RATE.to_le_bytes();
    let bytes_per_sec_bs = bytes_per_sec.to_le_bytes();
    let bytes_per_block_bs = bytes_per_block.to_le_bytes();
    let bits_per_sample_bs = BITS_PER_SAMPLE.to_le_bytes();
    let sample_data_len_bs = sample_data_len.to_le_bytes();
    let wav_header: [u8; 44] = [
        b'R', b'I', b'F', b'F',
        riff_size[0], riff_size[1], riff_size[2], riff_size[3],
        b'W', b'A', b'V', b'E',

        b'f', b'm', b't', b' ',
        0x10, 0x00, 0x00, 0x00, // "fmt " block size = 0x10
        0x01, 0x00, // audio format (1 = PCM integer)
        num_channels_bs[0], num_channels_bs[1], // number of channels
        sample_rate_bs[0], sample_rate_bs[1], sample_rate_bs[2], sample_rate_bs[3],
        bytes_per_sec_bs[0], bytes_per_sec_bs[1], bytes_per_sec_bs[2], bytes_per_sec_bs[3],
        bytes_per_block_bs[0], bytes_per_block_bs[1],
        bits_per_sample_bs[0], bits_per_sample_bs[1],

        b'd', b'a', b't', b'a',
        sample_data_len_bs[0], sample_data_len_bs[1], sample_data_len_bs[2], sample_data_len_bs[3],
    ];

    output_bs.splice(0..0, wav_header);
    std::fs::write(&opts.output_wav, &output_bs)
        .expect("failed to write output .wav file");
}
