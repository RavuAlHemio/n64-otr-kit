use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use otr_common::resource::ResourceHeader;
use otr_common::resource::resource_type;


#[derive(Parser)]
enum Mode {
    #[command(name = "as")]
    Assemble {
        /// The JSON input file which contains information about the sequence, including the path to
        /// the .seq file.
        info_json_output_file: PathBuf,

        /// The OTR SEQC resource file to write.
        seq_resource_output_file: PathBuf,
    },
    #[command(name = "dis")]
    Disassemble {
        /// The OTR SEQC resource file to read.
        seq_resource_input_file: PathBuf,

        /// The JSON output file in which to store information about the sequence.
        info_json_output_file: PathBuf,

        /// The .seq file into which to write the sequence data; this format can be further
        /// processed by sauraen's seq64.
        seq_output_file: PathBuf,
    },
}

fn main() {
    let mode = Mode::parse();
    match mode {
        Mode::Assemble {
            info_json_output_file,
            seq_resource_output_file,
        } => {
            todo!();
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
            todo!("import sequence sources from laptop");
        },
    }
}
