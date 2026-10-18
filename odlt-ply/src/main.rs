use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use clap::Parser;
use otr_archive::Otr2Archive;
use otr_common::display_list::{DisplayListV0Header, GraphicsCommand};
use otr_common::resource::{ResourceHeader};
use otr_common::resource::resource_type;
use otr_common::vertex::{VertexV0, VertexV0Header};


#[derive(Parser)]
struct Opts {
    /// The .o2r archive with the resources.
    pub input_o2r_file: PathBuf,

    /// The path of the display list file, within the .o2r file, to output.
    pub display_list_path: String,

    /// The path to which to write the output .ply file.
    pub output_ply_file: PathBuf,
}

fn main() {
    let opts = Opts::parse();
    let o2r_file = File::open(&opts.input_o2r_file)
        .expect("failed to open o2r file");
    let mut o2r = Otr2Archive::new(o2r_file)
        .expect("failed to load o2r archive");
    let display_list_bs = {
        let mut display_list_file = match o2r.as_mut().by_name(&opts.display_list_path) {
            Ok(dlf) => dlf,
            Err(e) => {
                panic!(
                    "failed to open display list file {:?} in o2r archive {:?}: {}",
                    opts.display_list_path, opts.input_o2r_file, e,
                );
            },
        };
        let mut buf = Vec::new();
        display_list_file.read_to_end(&mut buf)
            .expect("failed to read display list file from o2r archive");
        buf
    };

    // read the resource header
    let mut display_list_cursor = Cursor::new(&display_list_bs);
    let dl_resource_header = ResourceHeader::try_read(&mut display_list_cursor)
        .expect("failed to read display list resource header");
    if dl_resource_header.resource_type != resource_type::DISPLAY_LIST {
        panic!("unexpected resource type (obtained {:#010X}, expected {:#010X}", dl_resource_header.resource_type, resource_type::DISPLAY_LIST);
    }

    // read the header
    let dl_header = match dl_resource_header.version {
        0 => {
            let header = DisplayListV0Header::try_read(&mut display_list_cursor, dl_resource_header.byte_order)
                .expect("failed to read v0 display list header");
            header
        },
        other => panic!("unsupported display list version {}", other),
    };

    // crunch out the commands
    let commands = dl_header.try_read_data(&mut display_list_cursor, dl_resource_header.byte_order)
        .expect("failed to read display list graphics commands");

    // interpret them
    let mut vertex_buffer = Vec::new();
    let mut hash_to_known_vertices: BTreeMap<u64, Vec<VertexV0>> = BTreeMap::new();
    let mut triangles = Vec::with_capacity(0);
    for command in &commands {
        match command {
            GraphicsCommand::Vertex { v, n, v0 } => {
                panic!(
                    "cannot load {} vertices into vertex buffer at position {} because I don't know where memory location {:#010X} is",
                    n, v0, v,
                );
            },
            GraphicsCommand::Triangle1 { triangle_vertex_indexes, flag: _ } => {
                let mut vertexes = [VertexV0::default(); 3];
                for triangle_vertex_indexes in [triangle_vertex_indexes] {
                    for (tvi, vertex) in triangle_vertex_indexes.iter().copied().zip(vertexes.iter_mut()) {
                        let tvi_usize: usize = tvi.into();
                        let vertex_ref = vertex_buffer.get(tvi_usize)
                            .expect("triangle references vertex not in vertex buffer");
                        *vertex = *vertex_ref;
                    }
                    triangles.push(vertexes);
                }
            },
            GraphicsCommand::Triangle2 { triangle0_vertex_indexes, triangle1_vertex_indexes } => {
                let mut vertexes = [VertexV0::default(); 3];
                for triangle_vertex_indexes in [triangle0_vertex_indexes, triangle1_vertex_indexes] {
                    for (tvi, vertex) in triangle_vertex_indexes.iter().copied().zip(vertexes.iter_mut()) {
                        let tvi_usize: usize = tvi.into();
                        let vertex_ref = vertex_buffer.get(tvi_usize)
                            .expect("triangle references vertex not in vertex buffer");
                        *vertex = *vertex_ref;
                    }
                    triangles.push(vertexes);
                }
            },
            GraphicsCommand::EndDisplayList => {
                // yay
            },
            GraphicsCommand::OtrVertexByHash { v, n, v0, hash } => {
                // do we know the vertex definition?
                let vert_def = match hash_to_known_vertices.entry(hash.clone()) {
                    Entry::Vacant(vacant_entry) => {
                        // load it from the o2r
                        let vertex_bs = {
                            let mut vertex_file = match o2r.entry_by_crc64(*hash) {
                                Ok(vf) => vf,
                                Err(e) => {
                                    panic!("failed to open vertex definition with hash {:#018X}: {}", hash, e);
                                },
                            };
                            let mut vertex_bs = Vec::new();
                            vertex_file.read_to_end(&mut vertex_bs)
                                .expect("failed to read vertex definition");
                            vertex_bs
                        };
                        let mut v_cursor = Cursor::new(&vertex_bs);
                        let v_resource_header = ResourceHeader::try_read(&mut v_cursor)
                            .expect("failed to read vertex resource header");
                        if v_resource_header.resource_type != resource_type::VERTEX {
                            panic!(
                                "unexpected resource type (obtained {:#010X}, expected {:#010X}",
                                v_resource_header.resource_type, resource_type::VERTEX,
                            );
                        }
                        let v_header = match v_resource_header.version {
                            0 => {
                                let header = VertexV0Header::try_read(&mut v_cursor, v_resource_header.byte_order)
                                    .expect("failed to read v0 vertex header");
                                header
                            },
                            other => panic!("unsupported vertex version {}", other),
                        };
                        let mut vertices = Vec::new();
                        for _ in 0..v_header.vertex_count {
                            let vertex = VertexV0::try_read(&mut v_cursor, v_resource_header.byte_order)
                                .expect("failed to read vertex");
                            vertices.push(vertex);
                        }
                        vacant_entry.insert(vertices.clone());
                        vertices
                    },
                    Entry::Occupied(occupied_entry) => {
                        // sweet
                        occupied_entry.get().clone()
                    },
                };

                let v_usize: usize = (*v).try_into().unwrap();
                let n_usize: usize = (*n).into();
                let v0_usize: usize = (*v0).into();

                // expand vert_buffer
                if vertex_buffer.len() < v0_usize + n_usize {
                    vertex_buffer.reserve((v0_usize + n_usize) - vertex_buffer.len());
                }
                while vertex_buffer.len() < v0_usize + n_usize {
                    vertex_buffer.push(VertexV0::default());
                }

                // copy n vertices from vert_def[v..] to vertex_buffer[v0..]
                vertex_buffer[v0_usize..v0_usize+n_usize]
                    .copy_from_slice(&vert_def[v_usize..v_usize+n_usize]);
            },
        }
    }

    // fold vertices into references
    let mut vertex_to_id: BTreeMap<VertexV0, usize> = BTreeMap::new();
    let mut next_free_id = 0;
    let mut triangles_references = Vec::with_capacity(triangles.len());
    for triangle in &triangles {
        let mut references = [0usize; 3];
        for (vertex, reference) in triangle.iter().copied().zip(references.iter_mut()) {
            let new_ref = match vertex_to_id.entry(vertex) {
                Entry::Vacant(vacant_entry) => {
                    // the next one is yours!
                    let val = vacant_entry.insert(next_free_id);
                    next_free_id += 1;
                    *val
                },
                Entry::Occupied(occupied_entry) => {
                    // awesome, just reuse it
                    *occupied_entry.get()
                },
            };
            *reference = new_ref;
        }
        triangles_references.push(references);
    }

    // turn the lookup map into a list
    let mut vertices = vec![VertexV0::default(); vertex_to_id.len()];
    for (vertex, vertex_id) in vertex_to_id {
        vertices[vertex_id] = vertex;
    }

    // and now, we collect the whole thing as .ply
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
    write!(ply_buf, "element face {}\n", triangles_references.len()).unwrap();
    write!(ply_buf, "property list uchar uint vertex_indices\n").unwrap();
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

    for triangle_references in triangles_references {
        write!(ply_buf, "{}", triangle_references.len()).unwrap();
        for reference in triangle_references {
            write!(ply_buf, " {}", reference).unwrap();
        }
        write!(ply_buf, "\n").unwrap();
    }

    std::fs::write(&opts.output_ply_file, &ply_buf)
        .expect("failed to write out PLY file");
}
