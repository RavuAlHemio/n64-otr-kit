use crate::display_list::{Error, GbiVersion, GraphicsCommand, Operand};
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

fn decode_sp_no_op(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_sp_no_op(hi, lo)
}

fn decode_dp_no_op(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_dp_no_op(hi, lo)
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

fn decode_matrix(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    let (
        opcode,
        push,
        load,
        projection,
    ) = crate::display_list::shared::util_decode_pre2_matrix_opcode_push_load_projection(hi)?;
    assert_eq!(opcode, 0x01);
    let matrix = lo;

    Ok(GraphicsCommand::Matrix {
        matrix,
        push,
        load,
        projection,
    })
}

fn decode_fill_rectangle(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_fill_rectangle(hi, lo)
}

fn decode_set_geometry_mode(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_set_geometry_mode(hi, lo)
}

fn decode_clear_geometry_mode(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_clear_geometry_mode(hi, lo)
}

fn decode_texture(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_texture(hi, lo)
}

fn decode_set_primitive_color(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_primitive_color(hi, lo)
}

fn decode_set_environment_color(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_environment_color(hi, lo)
}

fn decode_set_fill_color(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_fill_color(hi, lo)
}

fn decode_set_texture_image(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_texture_image(hi, lo)
}

fn decode_otr_set_texture_image_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_otr_set_texture_image_by_hash(hi, lo, hash_hi, hash_lo)
}

fn decode_set_depth_image(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_depth_image(hi, lo)
}

fn decode_set_color_image(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_color_image(hi, lo)
}

fn decode_set_scissor_box(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_scissor_box(hi, lo)
}

fn decode_load_block(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_load_block(hi, lo)
}

fn decode_set_color_combine(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_color_combine(hi, lo)
}

fn decode_set_other_mode_lo(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_set_other_mode(false, hi, lo)
}

fn decode_set_other_mode_hi(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_set_other_mode(true, hi, lo)
}

fn decode_set_tile(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_tile(hi, lo)
}

fn decode_set_tile_size(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_tile_size(hi, lo)
}

fn decode_move_word(gbi_version: GbiVersion, hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_move_word(gbi_version, hi, lo)
}

fn decode_move_memory(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_move_memory(hi, lo)
}

fn decode_rdp_load_sync(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_rdp_load_sync(hi, lo)
}

fn decode_branch_display_list(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_branch_display_list(0x06, hi, lo)
}

fn decode_end_display_list(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_end_display_list(0xB8, hi, lo)
}

fn decode_otr_branch_display_list_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_otr_branch_display_list_by_hash(hi, lo, hash_hi, hash_lo)
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

fn decode_otr_matrix_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_pre2_matrix_opcode_push_load_projection
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    //
    // the asset is identified by hash_hi and hash_lo
    // (CRC64 of the file path within the archive)

    let (
        opcode,
        push,
        load,
        projection,
    ) = crate::display_list::shared::util_decode_pre2_matrix_opcode_push_load_projection(hi)?;
    assert_eq!(opcode, 0x36);
    let _ = lo;
    let hash =
        (u64::from(hash_hi) << 32)
        | (u64::from(hash_lo) << 0)
    ;
    Ok(GraphicsCommand::OtrMatrixByHash {
        hash,
        push,
        load,
        projection,
    })
}

fn decode_otr_move_memory(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_pre2_otr_move_memory(hi, lo, hash_hi, hash_lo)
}

pub fn take_next_command<I: Iterator<Item = u32>>(gbi_version: GbiVersion, source: &mut I) -> Result<Option<GraphicsCommand>, Error> {
    let Some(hi) = source.next() else { return Ok(None) };
    let lo = source.next()
        .ok_or(Error::SourceEndedEarly)?;

    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let command = match opcode {
        0x00 => decode_sp_no_op(hi, lo),
        0x01 => decode_matrix(hi, lo),
        0x03 => decode_move_memory(hi, lo),
        0x04 => decode_vertex(hi, lo),
        0x06 => decode_branch_display_list(hi, lo),
        0x20|0x31|0x32|0x36|0x42 => {
            // these ones need two more words
            let hash_hi = source.next()
                .ok_or(Error::SourceEndedEarly)?;
            let hash_lo = source.next()
                .ok_or(Error::SourceEndedEarly)?;
            match opcode {
                0x20 => decode_otr_set_texture_image_by_hash(hi, lo, hash_hi, hash_lo),
                0x31 => decode_otr_branch_display_list_by_hash(hi, lo, hash_hi, hash_lo),
                0x32 => decode_otr_vertex_by_hash(hi, lo, hash_hi, hash_lo),
                0x36 => decode_otr_matrix_by_hash(hi, lo, hash_hi, hash_lo),
                0x42 => decode_otr_move_memory(hi, lo, hash_hi, hash_lo),
                _ => unreachable!(),
            }
        },
        0xB6 => decode_clear_geometry_mode(hi, lo),
        0xB7 => decode_set_geometry_mode(hi, lo),
        0xB8 => decode_end_display_list(hi, lo),
        0xB9 => decode_set_other_mode_lo(hi, lo),
        0xBA => decode_set_other_mode_hi(hi, lo),
        0xBB => decode_texture(hi, lo),
        0xBC => decode_move_word(gbi_version, hi, lo),
        0xBF => decode_triangle1(hi, lo),
        0xC0 => decode_dp_no_op(hi, lo),
        0xE6 => decode_rdp_load_sync(hi, lo),
        0xED => decode_set_scissor_box(hi, lo),
        0xF2 => decode_set_tile_size(hi, lo),
        0xF3 => decode_load_block(hi, lo),
        0xF5 => decode_set_tile(hi, lo),
        0xF6 => decode_fill_rectangle(hi, lo),
        0xF7 => decode_set_fill_color(hi, lo),
        0xFA => decode_set_primitive_color(hi, lo),
        0xFB => decode_set_environment_color(hi, lo),
        0xFC => decode_set_color_combine(hi, lo),
        0xFD => decode_set_texture_image(hi, lo),
        0xFE => decode_set_depth_image(hi, lo),
        0xFF => decode_set_color_image(hi, lo),
        other => return Err(Error::UnsupportedOpcode { opcode: other }),
    }?;
    Ok(Some(command))
}
