use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use otr_common::resource::ResourceHeader;
use otr_common::resource::resource_type;
use otr_common::vertex::VertexV0;
use otr_common::vertex::VertexV0Header;


#[derive(Parser)]
struct Opts {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
}

fn main() {
    let opts = Opts::parse();

    // make a reader for the input
    let mut input_file = File::open(&opts.input_file)
        .expect("failed to open input file");

    // read the resource header
    let resource_header = ResourceHeader::try_read(&mut input_file)
        .expect("failed to read resource header");
    if resource_header.resource_type != resource_type::VERTEX {
        panic!("unexpected resource type (obtained {:#010X}, expected {:#010X}", resource_header.resource_type, resource_type::VERTEX);
    }

    // read the vertex header
    let texture_header = match resource_header.version {
        0 => {
            let header = VertexV0Header::try_read(&mut input_file, resource_header.byte_order)
                .expect("failed to read v0 vertex header");
            header
        },
        other => panic!("unsupported vertex version {}", other),
    };

    // read the vertices
    let vertex_count: usize = texture_header.vertex_count
        .try_into().expect("failed to convert vertex count to usize");
    let mut vertices = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let vertex = VertexV0::try_read(&mut input_file, resource_header.byte_order)
            .expect("failed to read vertex");
        vertices.push(vertex);
    }

    // output them as .ply
    let mut ply_buf = Vec::new();
    write!(ply_buf, "ply\n").unwrap();
    write!(ply_buf, "format ascii 1.0\n").unwrap();
    write!(ply_buf, "element vertex {}\n", vertices.len()).unwrap();
    write!(ply_buf, "property int16 x\n").unwrap();
    write!(ply_buf, "property int16 y\n").unwrap();
    write!(ply_buf, "property int16 z\n").unwrap();
    write!(ply_buf, "property uint16 flag\n").unwrap();
    write!(ply_buf, "property int16 s\n").unwrap();
    write!(ply_buf, "property int16 t\n").unwrap();
    write!(ply_buf, "property uint8 nx\n").unwrap();
    write!(ply_buf, "property uint8 ny\n").unwrap();
    write!(ply_buf, "property uint8 nz\n").unwrap();
    write!(ply_buf, "property uint8 red\n").unwrap();
    write!(ply_buf, "property uint8 green\n").unwrap();
    write!(ply_buf, "property uint8 blue\n").unwrap();
    write!(ply_buf, "property uint8 alpha\n").unwrap();
    write!(ply_buf, "end_header\n").unwrap();

    for vertex in &vertices {
        write!(
            ply_buf,
            "{} {} {} {} {} {} {} {} {} {} {} {} {}\n",
            vertex.x, vertex.y, vertex.z,
            vertex.flag,
            vertex.texture_s, vertex.texture_t,
            vertex.red_or_normal_x, vertex.green_or_normal_y, vertex.blue_or_normal_z,
            vertex.red_or_normal_x, vertex.green_or_normal_y, vertex.blue_or_normal_z, vertex.alpha,
        ).unwrap();
    }

    std::fs::write(&opts.output_file, &ply_buf)
        .expect("failed to write out PLY file");
}
