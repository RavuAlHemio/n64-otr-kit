use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use otr_common::common::Boolean;
use otr_common::common::Endianness;
use otr_common::resource::ResourceHeader;
use otr_common::resource::resource_type;
use otr_common::sound::sequence::SequenceV0Header;
use serde::{Deserialize, Serialize};


#[derive(Parser)]
enum Mode {
    /// Assemble an OTR sequence resource file from the given JSON info file (and the sequence data
    /// file it references).
    #[command(name = "as")]
    Assemble {
        /// The JSON input file which contains information about the sequence, including the path to
        /// the .seq file.
        info_json_output_file: PathBuf,

        /// The OTR SEQC resource file to write.
        seq_resource_output_file: PathBuf,
    },
    /// Disassemble an OTR sequence resource file into a JSON info file and a .seq data file.
    #[command(name = "dis")]
    Disassemble {
        /// The OTR SEQC resource file to read.
        seq_resource_input_file: PathBuf,

        /// The JSON output file in which to store information about the sequence.
        info_json_output_file: PathBuf,

        /// The .seq file into which to write the sequence data; this format can be further
        /// processed by sauraen's seq64.
        seq_output_file: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct SequenceInfo {
    pub big_endian: bool,
    pub custom: bool,
    pub version: u32,
    pub id: u32,
    pub banks: Vec<String>,
    pub data_file_path: String,
}

fn main() {
    let mode = Mode::parse();
    match mode {
        Mode::Assemble {
            info_json_output_file,
            seq_resource_output_file,
        } => {
            let input_string = std::fs::read_to_string(&info_json_output_file)
                .expect("failed to read JSON sequence description input file");
            let input_info: SequenceInfo = serde_json::from_str(&input_string)
                .expect("failed to parse JSON sequence description");
            if input_info.version != 0 {
                panic!("don't know how to handle sequence versions other than 0");
            }

            // load the sequence data
            let seq_data = std::fs::read(&input_info.data_file_path)
                .expect("failed to load sequence data");

            // assemble the resource header
            let resource_header = ResourceHeader {
                byte_order: if input_info.big_endian { Endianness::Big } else { Endianness::Little },
                is_custom: Boolean::from(input_info.custom),
                padding0: [0, 0],
                resource_type: resource_type::SEQUENCE,
                version: 0,
                asset_id: 0xDEADBEEF_DEADBEEF,
                padding1: [0u8; 44],
            };

            // assemble the sequence header
            let sequence_header = SequenceV0Header {
                id: input_info.id,
                banks: input_info.banks,
                data_byte_count: seq_data.len().try_into().unwrap(),
            };

            // write it all out
            let mut seq_resource_buf = Vec::new();
            let mut seq_resource_cur = Cursor::new(&mut seq_resource_buf);
            resource_header.write(&mut seq_resource_cur)
                .expect("failed to serialize resource header");
            sequence_header.write(&mut seq_resource_cur, resource_header.byte_order)
                .expect("failed to serialize sequence header");
            seq_resource_cur.write(&seq_data)
                .expect("failed to copy sequence data");

            std::fs::write(&seq_resource_output_file, &seq_resource_buf)
                .expect("failed to write out assembled resource file");
        },
        Mode::Disassemble {
            seq_resource_input_file,
            info_json_output_file,
            seq_output_file,
        } => {
            let mut input_file = File::open(&seq_resource_input_file)
                .expect("failed to open sequence resource input file");

            // read the resource header
            let resource_header = ResourceHeader::try_read(&mut input_file)
                .expect("failed to read resource header");
            if resource_header.resource_type != resource_type::SEQUENCE {
                panic!(
                    "input resource has type {:#010X}; expected {:#010X}",
                    resource_header.resource_type, resource_type::SEQUENCE,
                );
            }
            if resource_header.version != 0 {
                panic!("unsupported sequence version {}", resource_header.version);
            }

            // read the sequence header
            let seq_header = SequenceV0Header::try_read(&mut input_file, resource_header.byte_order)
                .expect("failed to read sequence header");
            let seq_data_len: usize = seq_header.data_byte_count.try_into().unwrap();

            // read the sequence data
            let mut seq_data_buf = vec![0u8; seq_data_len];
            input_file.read_exact(&mut seq_data_buf)
                .expect("failed to read sequence data");

            // assemble the sequence info
            let seq_info = SequenceInfo {
                big_endian: resource_header.byte_order == Endianness::Big,
                custom: resource_header.is_custom.as_bool(),
                version: 0,
                id: seq_header.id,
                banks: seq_header.banks,
                data_file_path: seq_output_file.clone(),
            };
            let seq_info_string = serde_json::to_string_pretty(&seq_info)
                .expect("failed to serialize sequence info as JSON");
            std::fs::write(&info_json_output_file, &seq_info_string)
                .expect("failed to write metadata JSON");

            std::fs::write(&seq_output_file, &seq_data_buf)
                .expect("failed to write sequence data buffer");
        },
    }
}
