use crate::display_list::{Error, GraphicsCommand, Operand};
use crate::display_list::shared::util_scale_triangle_vector_indexes;


fn util_decode_vertex_opcode_n_v0(hi: u32) -> Result<(u8, u8, u8), Error> {
    // hi: OOOO OOOO xxxx NNNN NNNN xxxx LLLL LLLx
    // O = opcode, N = number of vertices, L = index after last vertex in the target buffer (v0)

    // we are opcode-agnostic; let the decode_* functions verify it
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let n = u8::try_from((hi >> 12) & 0xFF).unwrap();
    let vl = u8::try_from((hi >> 1) & 0x7F).unwrap();
    let v0 = vl.checked_sub(n)
        .ok_or(Error::SubtractionUnderflow { minuend: Operand::VLast, subtrahend: Operand::N })?;
    Ok((opcode, n, v0))
}

fn decode_vertex(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_vertex_n_v0
    // lo: VVVV VVVV VVVV VVVV VVVV VVVV VVVV VVVV
    // V = vertex data pointer

    let (opcode, n, v0) = util_decode_vertex_opcode_n_v0(hi)?;
    assert_eq!(opcode, 0x01);
    let v = lo;
    Ok(GraphicsCommand::Vertex {
        v,
        n,
        v0,
    })
}

fn decode_triangle1(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO AAAA AAAA BBBB BBBB CCCC CCCC
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // O = opcode, A through C = indexes into the vertex buffer multiplied by 2
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0x05);
    let _ = lo;
    let vi_raw = [
        u8::try_from((hi >> 16) & 0xFF).unwrap(),
        u8::try_from((hi >>  8) & 0xFF).unwrap(),
        u8::try_from((hi >>  0) & 0xFF).unwrap(),
    ];
    let triangle_vertex_indexes = util_scale_triangle_vector_indexes(
        0,
        vi_raw,
        2,
    )?;
    Ok(GraphicsCommand::Triangle1 {
        triangle_vertex_indexes,
        flag: None,
    })
}

fn decode_triangle2(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_triangle2(0x06, hi, lo)
}

fn decode_end_display_list(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_end_display_list(0xDF, hi, lo)
}

fn decode_otr_vertex_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_vertex_n_v0
    // lo: VVVV VVVV VVVV VVVV VVVV VVVV VVVV VVVV
    // V = byte offset within the vertex table of the asset
    // (V must be a multiple of sizeof(Vtx), where sizeof(Vtx) == 16)
    //
    // the asset is identified by hash_hi and hash_lo
    // (CRC64 of the file path within the archive)

    let (opcode, n, v0) = util_decode_vertex_opcode_n_v0(hi)?;
    assert_eq!(opcode, 0x32);
    let v_raw = lo;
    Error::ensure_operand_multiple(
        Operand::VOtr,
        16,
        v_raw.into(),
    )?;
    let v = v_raw / 16;
    let hash =
        (u64::from(hash_hi) << 32)
        | (u64::from(hash_lo) << 0)
    ;
    Ok(GraphicsCommand::OtrVertexByHash {
        v,
        n,
        v0,
        hash,
    })
}

pub fn take_next_command<I: Iterator<Item = u32>>(source: &mut I) -> Result<Option<GraphicsCommand>, Error> {
    let Some(hi) = source.next() else { return Ok(None) };
    let lo = source.next()
        .ok_or(Error::SourceEndedEarly)?;

    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let command = match opcode {
        0x01 => decode_vertex(hi, lo),
        0x05 => decode_triangle1(hi, lo),
        0x06 => decode_triangle2(hi, lo),
        0x32 => {
            // this one needs two more words
            let hash_hi = source.next()
                .ok_or(Error::SourceEndedEarly)?;
            let hash_lo = source.next()
                .ok_or(Error::SourceEndedEarly)?;
            decode_otr_vertex_by_hash(hi, lo, hash_hi, hash_lo)
        },
        0xDF => decode_end_display_list(hi, lo),
        other => return Err(Error::UnsupportedOpcode { opcode: other }),
    }?;
    Ok(Some(command))
}
