use crate::display_list::{Error, GraphicsCommand, Operand};


pub fn decode_end_display_list(expected_opcode: u8, hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // O = opcode
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let _ = lo;
    assert_eq!(opcode, expected_opcode);
    Ok(GraphicsCommand::EndDisplayList)
}

pub fn decode_triangle2(expected_opcode: u8, hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO AAAA AAAA BBBB BBBB CCCC CCCC
    // lo: xxxx xxxx DDDD DDDD EEEE EEEE FFFF FFFF
    // O = opcode, A through F = indexes into the vertex buffer multiplied by 2
    // (A through C for triangle 0, D through F for triangle 1)
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, expected_opcode);
    let t0vi_raw = [
        u8::try_from((hi >> 16) & 0xFF).unwrap(),
        u8::try_from((hi >>  8) & 0xFF).unwrap(),
        u8::try_from((hi >>  0) & 0xFF).unwrap(),
    ];
    let t1vi_raw = [
        u8::try_from((lo >> 16) & 0xFF).unwrap(),
        u8::try_from((lo >>  8) & 0xFF).unwrap(),
        u8::try_from((lo >>  0) & 0xFF).unwrap(),
    ];
    let triangle0_vertex_indexes = util_scale_triangle_vector_indexes(
        0,
        t0vi_raw,
        2,
    )?;
    let triangle1_vertex_indexes = util_scale_triangle_vector_indexes(
        1,
        t1vi_raw,
        2,
    )?;
    Ok(GraphicsCommand::Triangle2 {
        triangle0_vertex_indexes,
        triangle1_vertex_indexes,
    })
}

pub fn util_scale_triangle_vector_indexes(
    triangle_index: u8,
    raw_indexes: [u8; 3],
    divisor: u8,
) -> Result<[u8; 3], Error> {
    for (i, vi) in raw_indexes.iter().enumerate() {
        let iu8 = u8::try_from(i).unwrap();
        Error::ensure_operand_multiple(
            Operand::TriangleVertexIndex {
                triangle_index,
                vertex_index: iu8,
            },
            u64::from(divisor),
            u64::from(*vi),
        )?;
    }
    Ok([
        raw_indexes[0] / divisor,
        raw_indexes[1] / divisor,
        raw_indexes[2] / divisor,
    ])
}
