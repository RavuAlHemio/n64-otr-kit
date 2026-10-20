use crate::display_list::{Error, GbiVersion, GraphicsCommand, Operand};
use crate::display_list::move_word::MoveWordTarget;
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

fn util_decode_matrix_opcode_push_load_projection(hi: u32) -> Result<(u8, bool, bool, bool), Error> {
    // and another move from nicely organized operands in F3DEX to utter chaos in F3DEX2...
    // hi: OOOO OOOO SSSS Sxxx xxxx xxxx xxxx xJLR
    // O = opcode, R = replace (not push), L = load (not multiply), J = proJection (not model view),
    // S = size value
    //
    // S must be equal to (sizeof(Mtx) - 1) / 8, which is (64 - 1) / 8, which is 7

    // we are opcode-agnostic; let the decode_* functions verify it
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();

    let replace = ((hi >> 0) & 0b1) != 0;
    let push = !replace;
    let load = ((hi >> 1) & 0b1) != 0;
    let projection = ((hi >> 2) & 0b1) != 0;

    let size_value = u16::try_from((hi >> 19) & 0b11111).unwrap();
    if size_value != 7 {
        return Err(Error::SizeMismatch {
            calculated: 7,
            stored: usize::from(size_value),
        });
    }

    Ok((opcode, push, load, projection))
}

fn util_decode_set_other_mode(upper_half: bool, hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx SSSS SSSS LLLL LLLL
    // lo: DDDD DDDD DDDD DDDD DDDD DDDD DDDD DDDD
    // O = opcode, S = shift spec, L = length spec, D = data
    // such that
    // length = length_spec + 1
    // shift = 32 - (shift_spec + length)
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let expected_opcode = if upper_half { 0xE3 } else { 0xE2 };
    assert_eq!(opcode, expected_opcode);
    let shift_spec = u8::try_from((hi >> 8) & 0xFF).unwrap();
    let length_spec = u8::try_from((hi >> 0) & 0xFF).unwrap();
    let data = lo;

    let length = length_spec + 1;
    let shift = 32 - (shift_spec + length);

    Ok(GraphicsCommand::SetOtherMode {
        upper_half,
        shift,
        length,
        data,
    })
}

pub fn util_decode_move_memory_opcode_index_offset_size(hi: u32) -> Result<(u8, u8, u16, u16), Error> {
    // hi: OOOO OOOO SSSS Sxxx OOOO OOOO IIII IIII
    // O = opcode, I = index, S = size value, T = offset value
    // size = (size_value + 1) * 8
    // offset = offset_value * 8

    // we are opcode-agnostic; let the decode_* functions verify it
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let index = u8::try_from((hi >> 0) & 0xFF).unwrap();
    let offset_value = u16::try_from((hi >> 8) & 0x00FF).unwrap();
    let size_value = u16::try_from((hi >> 19) & 0b11111).unwrap();

    let size = (size_value + 1) * 8;
    let offset = offset_value * 8;

    Ok((opcode, index, offset, size))
}

fn decode_sp_no_op(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // O = opcode
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xE0);
    let _ = lo;
    Ok(GraphicsCommand::SpNoOp)
}

fn decode_dp_no_op(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // O = opcode
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0x00);
    let _ = lo;
    Ok(GraphicsCommand::DpNoOp)
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

fn decode_matrix(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_matrix_opcode_push_load_projection
    // lo: MMMM MMMM MMMM MMMM MMMM MMMM MMMM MMMM
    // M = matrix location in memory

    let (
        opcode,
        push,
        load,
        projection,
    ) = util_decode_matrix_opcode_push_load_projection(hi)?;
    assert_eq!(opcode, 0xDA);
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

fn decode_geometry_mode(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO CCCC CCCC CCCC CCCC CCCC CCCC
    // lo: MMMM MMMM MMMM MMMM MMMM MMMM MMMM MMMM
    // O = opcode, C = mode bits *not* to clear, M = mode bits to set
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xD9);
    let clear_bits = ((!hi) >> 0) & 0x00FFFFFF;
    Ok(GraphicsCommand::GeometryMode {
        set_bits: lo,
        clear_bits,
    })
}

fn decode_texture(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxLL LNNN AAAA AAAx
    // lo: SSSS SSSS SSSS SSSS TTTT TTTT TTTT TTTT
    // O = opcode, L = max mipmap level, N = tile number, A = on (active) flag,
    // S and T = texture coordinate scaling (fractional part)

    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xD7);
    let max_mipmap_levels = u8::try_from((hi >> 11) & 0b111).unwrap();
    let tile_number = u8::try_from((hi >> 8) & 0b111).unwrap();
    let on = ((hi >> 1) & 0x7F) != 0;
    let s_scale = u16::try_from((lo >> 16) & 0xFFFF).unwrap();
    let t_scale = u16::try_from((lo >> 0) & 0xFFFF).unwrap();
    Ok(GraphicsCommand::TexturingParameters {
        s_scale,
        t_scale,
        max_mipmap_levels,
        tile_number,
        on,
    })
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
    util_decode_set_other_mode(false, hi, lo)
}

fn decode_set_other_mode_hi(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    util_decode_set_other_mode(true, hi, lo)
}

fn decode_set_tile(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_tile(hi, lo)
}

fn decode_set_tile_size(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_set_tile_size(hi, lo)
}

fn decode_move_word(gbi_version: GbiVersion, hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO IIII IIII PPPP PPPP PPPP PPPP
    // lo: DDDD DDDD DDDD DDDD DDDD DDDD DDDD DDDD
    // O = opcode, P = offset (position), I = index, D = data

    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xDB);
    let index = u8::try_from((hi >> 16) & 0xFF).unwrap();
    let offset = u16::try_from((hi >> 0) & 0xFFFF).unwrap();
    let data = lo;

    let target = MoveWordTarget::from_index_offset_data(gbi_version, index, offset, data);

    Ok(GraphicsCommand::MoveWord {
        target,
        data,
    })
}

pub fn decode_move_memory(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_move_memory_opcode_index_offset_size
    // lo: MMMM MMMM MMMM MMMM MMMM MMMM MMMM MMMM
    // M = memory location

    let (
        opcode,
        index,
        offset,
        size,
    ) = util_decode_move_memory_opcode_index_offset_size(hi)?;
    assert_eq!(opcode, 0xDC);
    let source_address = lo;

    Ok(GraphicsCommand::MoveMemory {
        size,
        index,
        offset: Some(offset),
        source_address,
    })
}

fn decode_rdp_pipe_sync(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // O = opcode
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let _ = lo;
    assert_eq!(opcode, 0xE7);
    Ok(GraphicsCommand::RdpPipeSync)
}

fn decode_rdp_load_sync(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_rdp_load_sync(hi, lo)
}

fn decode_branch_display_list(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_branch_display_list(0xDE, hi, lo)
}

fn decode_end_display_list(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_end_display_list(0xDF, hi, lo)
}

fn decode_otr_branch_display_list_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    crate::display_list::shared::decode_otr_branch_display_list_by_hash(hi, lo, hash_hi, hash_lo)
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

fn decode_otr_matrix_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_matrix_opcode_push_load_projection
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    //
    // the asset is identified by hash_hi and hash_lo
    // (CRC64 of the file path within the archive)

    let (
        opcode,
        push,
        load,
        projection,
    ) = util_decode_matrix_opcode_push_load_projection(hi)?;
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
    // hi: see util_decode_move_memory_opcode_index_offset_size
    // lo: IIII IIII TTTT TTTT HHHH HHHH xxxx xxxx
    // I = index for OTR, T = offset for OTR, H = "has offset?" for OTR (0 or 1)
    //
    // the asset is identified by hash_hi and hash_lo
    // (CRC64 of the file path within the archive)

    let (
        opcode,
        index,
        offset,
        size,
    ) = util_decode_move_memory_opcode_index_offset_size(hi)?;
    assert_eq!(opcode, 0x42);
    let otr_index = u8::try_from((lo >> 24) & 0xFF).unwrap();
    let otr_offset = u16::try_from((lo >> 16) & 0xFF).unwrap();
    let otr_has_offset = ((lo >> 8) & 0xFF) != 0;
    let hash =
        (u64::from(hash_hi) << 32)
        | (u64::from(hash_lo) << 0)
    ;
    Ok(GraphicsCommand::OtrMoveMemoryByHash {
        size,
        index,
        offset: Some(offset),
        otr_index,
        otr_offset,
        otr_has_offset,
        hash,
    })
}

pub fn take_next_command<I: Iterator<Item = u32>>(gbi_version: GbiVersion, source: &mut I) -> Result<Option<GraphicsCommand>, Error> {
    let Some(hi) = source.next() else { return Ok(None) };
    let lo = source.next()
        .ok_or(Error::SourceEndedEarly)?;

    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let command = match opcode {
        0x00 => decode_dp_no_op(hi, lo),
        0x01 => decode_vertex(hi, lo),
        0x03 => decode_move_memory(hi, lo),
        0x05 => decode_triangle1(hi, lo),
        0x06 => decode_triangle2(hi, lo),
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
        0xD7 => decode_texture(hi, lo),
        0xD9 => decode_geometry_mode(hi, lo),
        0xDA => decode_matrix(hi, lo),
        0xDB => decode_move_word(gbi_version, hi, lo),
        0xDE => decode_branch_display_list(hi, lo),
        0xDF => decode_end_display_list(hi, lo),
        0xE0 => decode_sp_no_op(hi, lo),
        0xE2 => decode_set_other_mode_lo(hi, lo),
        0xE3 => decode_set_other_mode_hi(hi, lo),
        0xE6 => decode_rdp_load_sync(hi, lo),
        0xE7 => decode_rdp_pipe_sync(hi, lo),
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
