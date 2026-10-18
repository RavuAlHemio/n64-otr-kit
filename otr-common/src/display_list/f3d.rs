use crate::display_list::{Error, GraphicsCommand, Operand};
use crate::display_list::shared::util_scale_triangle_vector_indexes;


fn util_decode_vertex_opcode_n_v0_size(hi: u32) -> Result<(u8, u8, u8, u16), Error> {
    // hi: OOOO OOOO NNNN TTTT SSSS SSSS SSSS SSSS
    // O = opcode, N = number of vertices, T = target offset (v0), S = size
    // S must equal sizeof(Vtx) * N, where sizeof(Vtx) == 16

    // we are opcode-agnostic; let the decode_* functions verify it
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let n = u8::try_from((hi >> 20) & 0x0F).unwrap();
    let v0 = u8::try_from((hi >> 16) & 0x0F).unwrap();
    let size = u16::try_from((hi >> 0) & 0xFFFF).unwrap();
    if size != u16::from(n) * 16 {
        return Err(Error::SizeMismatch {
            calculated: usize::from(n) * 16,
            stored: usize::from(size),
        });
    }
    Ok((opcode, n, v0, size))
}

fn decode_vertex(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_vertex_n_v0_size
    // lo: VVVV VVVV VVVV VVVV VVVV VVVV VVVV VVVV
    // V = vertex pointer

    let (opcode, n, v0, _size) = util_decode_vertex_opcode_n_v0_size(hi)?;
    assert_eq!(opcode, 0x04);
    let v = lo;
    Ok(GraphicsCommand::Vertex {
        v,
        n,
        v0,
    })
}

fn decode_triangle1(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: FFFF FFFF AAAA AAAA BBBB BBBB CCCC CCCC
    // O = opcode, A through C = indexes into the vertex buffer multiplied by 10, F = flag
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xBF);
    let flag = Some(u8::try_from((lo >> 24) & 0xFF).unwrap());
    let vi_raw = [
        u8::try_from((lo >> 16) & 0xFF).unwrap(),
        u8::try_from((lo >>  8) & 0xFF).unwrap(),
        u8::try_from((lo >>  0) & 0xFF).unwrap(),
    ];
    let triangle_vertex_indexes = util_scale_triangle_vector_indexes(
        0,
        vi_raw,
        10,
    )?;
    Ok(GraphicsCommand::Triangle1 {
        triangle_vertex_indexes,
        flag,
    })
}

fn decode_end_display_list(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_end_display_list(0xB8, hi, lo)
}

fn decode_otr_vertex_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_vertex_n_v0_size
    // lo: VVVV VVVV VVVV VVVV VVVV VVVV VVVV VVVV
    // V = byte offset within the vertex table of the asset
    // (V must be a multiple of sizeof(Vtx), where sizeof(Vtx) == 16)
    //
    // the asset is identified by hash_hi and hash_lo
    // (CRC64 of the file path within the archive)

    let (opcode, n, v0, _size) = util_decode_vertex_opcode_n_v0_size(hi)?;
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
        0x04 => decode_vertex(hi, lo),
        0x32 => {
            // this one needs two more words
            let hash_hi = source.next()
                .ok_or(Error::SourceEndedEarly)?;
            let hash_lo = source.next()
                .ok_or(Error::SourceEndedEarly)?;
            decode_otr_vertex_by_hash(hi, lo, hash_hi, hash_lo)
        },
        0xB8 => decode_end_display_list(hi, lo),
        0xBF => decode_triangle1(hi, lo),
        other => return Err(Error::UnsupportedOpcode { opcode: other }),
    }?;
    Ok(Some(command))
}
