use crate::display_list::{BitsPerPixel, BranchFlag, ColorFormat, Error, GraphicsCommand, Operand};
use crate::display_list::color_source::ColorSource;


pub fn decode_branch_display_list(expected_opcode: u8, hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO FFFF FFFF xxxx xxxx xxxx xxxx
    // lo: DDDD DDDD DDDD DDDD DDDD DDDD DDDD DDDD
    // O = opcode, F = branch flag, D = display list memory address
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, expected_opcode);
    let branch_flag_u8 = u8::try_from((hi >> 16) & 0xFF).unwrap();
    let branch_flag = BranchFlag::from_base_type(branch_flag_u8);
    let display_list = lo;
    Ok(GraphicsCommand::BranchDisplayList {
        display_list,
        branch_flag,
    })
}

pub fn decode_otr_branch_display_list_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO FFFF FFFF xxxx xxxx xxxx xxxx
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // O = opcode, F = branch flag
    // display list is in the asset identified by hash_hi and hash_lo
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0x31);
    let branch_flag_u8 = u8::try_from((hi >> 16) & 0xFF).unwrap();
    let branch_flag = BranchFlag::from_base_type(branch_flag_u8);
    let _ = lo;
    let hash =
        (u64::from(hash_hi) << 32)
        | (u64::from(hash_lo) << 0)
    ;
    Ok(GraphicsCommand::OtrBranchDisplayListByHash {
        branch_flag,
        hash,
    })
}

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

pub fn util_decode_pre2_matrix_opcode_push_load_projection(hi: u32) -> Result<(u8, bool, bool, bool), Error> {
    // hi: OOOO OOOO xxxx xPLJ SSSS SSSS SSSS SSSS
    // O = opcode, P = push (not replace), L = load (not multiply), J = proJection (not model view),
    // S = size of matrix
    //
    // S must be equal to sizeof(Mtx), which is 64

    // we are opcode-agnostic; let the decode_* functions verify it
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();

    let push = ((hi >> 18) & 0b1) != 0;
    let load = ((hi >> 17) & 0b1) != 0;
    let projection = ((hi >> 16) & 0b1) != 0;
    let size = u16::try_from((hi >> 0) & 0xFFFF).unwrap();
    if size != 64 {
        return Err(Error::SizeMismatch {
            calculated: 64,
            stored: usize::from(size),
        });
    }

    Ok((opcode, push, load, projection))
}

pub fn decode_set_primitive_color(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx MMMM MMMM LLLL LLLL
    // lo: RRRR RRRR GGGG GGGG BBBB BBBB AAAA AAAA
    // O = opcode, M = minimum value clamp for LOD, L = LOD fraction
    // R = red, G = green, B = blue, A = alpha
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xFA);
    let lod_minimum_value_clamp = u8::try_from((hi >> 8) & 0xFF).unwrap();
    let lod_fraction = u8::try_from((hi >> 0) & 0xFF).unwrap();
    let red = u8::try_from((lo >> 24) & 0xFF).unwrap();
    let green = u8::try_from((lo >> 16) & 0xFF).unwrap();
    let blue = u8::try_from((lo >> 8) & 0xFF).unwrap();
    let alpha = u8::try_from((lo >> 0) & 0xFF).unwrap();
    Ok(GraphicsCommand::SetPrimitiveColor {
        lod_minimum_value_clamp,
        lod_fraction,
        red,
        green,
        blue,
        alpha,
    })
}

pub fn decode_set_color_combine(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO AAAA CCCC CEEE GGGI IIIK KKKK
    // lo: BBBB JJJJ MMMP PPDD DFFF HHHL LLNN NQQQ
    // O = opcode, ABCD = first-cycle colors a-d, EFGH = first-cycle alpha a-d,
    // IJKL = second-cycle colors a-d, MNPQ = second-cycle alpha a-d
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xFC);
    let a = u8::try_from((hi >> 20) & 0b1111).unwrap();
    let b = u8::try_from((lo >> 28) & 0b1111).unwrap();
    let c = u8::try_from((hi >> 15) & 0b1_1111).unwrap();
    let d = u8::try_from((lo >> 15) & 0b111).unwrap();
    let e = u8::try_from((hi >> 12) & 0b111).unwrap();
    let f = u8::try_from((lo >> 12) & 0b111).unwrap();
    let g = u8::try_from((hi >>  9) & 0b111).unwrap();
    let h = u8::try_from((lo >>  9) & 0b111).unwrap();
    let i = u8::try_from((hi >>  5) & 0b1111).unwrap();
    let j = u8::try_from((lo >> 24) & 0b1111).unwrap();
    let k = u8::try_from((hi >>  0) & 0b1_1111).unwrap();
    let l = u8::try_from((lo >>  6) & 0b111).unwrap();
    let m = u8::try_from((lo >> 21) & 0b111).unwrap();
    let n = u8::try_from((lo >>  3) & 0b111).unwrap();
    let p = u8::try_from((lo >> 18) & 0b111).unwrap();
    let q = u8::try_from((lo >>  0) & 0b111).unwrap();

    Ok(GraphicsCommand::SetColorCombine {
        cycle_0_color_a: ColorSource::decode_color_a(a).unwrap(),
        cycle_0_color_b: ColorSource::decode_color_b(b).unwrap(),
        cycle_0_color_c: ColorSource::decode_color_c(c).unwrap(),
        cycle_0_color_d: ColorSource::decode_color_d(d).unwrap(),
        cycle_0_alpha_a: ColorSource::decode_alpha_a_b_d(e).unwrap(),
        cycle_0_alpha_b: ColorSource::decode_alpha_a_b_d(f).unwrap(),
        cycle_0_alpha_c: ColorSource::decode_alpha_c(g).unwrap(),
        cycle_0_alpha_d: ColorSource::decode_alpha_a_b_d(h).unwrap(),
        cycle_1_color_a: ColorSource::decode_color_a(i).unwrap(),
        cycle_1_color_b: ColorSource::decode_color_b(j).unwrap(),
        cycle_1_color_c: ColorSource::decode_color_c(k).unwrap(),
        cycle_1_color_d: ColorSource::decode_color_d(l).unwrap(),
        cycle_1_alpha_a: ColorSource::decode_alpha_a_b_d(m).unwrap(),
        cycle_1_alpha_b: ColorSource::decode_alpha_a_b_d(n).unwrap(),
        cycle_1_alpha_c: ColorSource::decode_alpha_c(p).unwrap(),
        cycle_1_alpha_d: ColorSource::decode_alpha_a_b_d(q).unwrap(),
    })
}

pub fn decode_set_fill_color(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: CCCC CCCC CCCC CCCC CCCC CCCC CCCC CCCC
    // O = opcode, C = color
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xF7);
    let color = lo;

    Ok(GraphicsCommand::SetFillColor {
        color,
    })
}

pub fn decode_set_environment_color(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: RRRR RRRR GGGG GGGG BBBB BBBB AAAA AAAA
    // O = opcode, R = red, G = green, B = blue, A = alpha
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xFB);
    let red = u8::try_from((lo >> 24) & 0xFF).unwrap();
    let green = u8::try_from((lo >> 16) & 0xFF).unwrap();
    let blue = u8::try_from((lo >>  8) & 0xFF).unwrap();
    let alpha = u8::try_from((lo >>  0) & 0xFF).unwrap();

    Ok(GraphicsCommand::SetEnvironmentColor {
        red,
        green,
        blue,
        alpha,
    })
}

pub fn decode_set_texture_image(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_set_texture_image_opcode_format_bpp_width
    // lo: IIII IIII IIII IIII IIII IIII IIII IIII
    // I = image address
    let (
        opcode,
        color_format,
        bits_per_pixel,
        width,
    ) = util_decode_set_texture_image_opcode_format_bpp_width(hi)?;
    assert_eq!(opcode, 0xFD);
    let image_address = lo;

    Ok(GraphicsCommand::SetTextureImage {
        color_format,
        bits_per_pixel,
        width,
        image_address,
    })
}

pub fn decode_otr_set_texture_image_by_hash(hi: u32, lo: u32, hash_hi: u32, hash_lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: see util_decode_set_texture_image_opcode_format_bpp_width
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // image is in the asset identified by hash_hi and hash_lo
    let (
        opcode,
        color_format,
        bits_per_pixel,
        width,
    ) = util_decode_set_texture_image_opcode_format_bpp_width(hi)?;
    assert_eq!(opcode, 0x20);
    let _ = lo;
    let hash =
        (u64::from(hash_hi) << 32)
        | (u64::from(hash_lo) << 0)
    ;

    Ok(GraphicsCommand::OtrSetTextureImageByHash {
        color_format,
        bits_per_pixel,
        width,
        hash,
    })
}

pub fn decode_set_tile(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO FFFI IxNN NNNN NNNM MMMM MMMM
    // lo: xxxx xTTT PPPP CCAA AASS SSDD BBBB UUUU
    // O = opcode, F = color format, I = bits per pixel, N = values per row, M = tmem offset
    // T = tile descriptor, P = palette index,
    // C = clamp (2) and mirror (1) flags in T, A = mask T, S = shift T,
    // D = clamp (2) and mirror (1) flags in S, B = mask S, U = shift S
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xF5);
    let color_format_u8 = u8::try_from((hi >> 21) & 0b111).unwrap();
    let bits_per_pixel_u8 = u8::try_from((hi >> 19) & 0b11).unwrap();
    let values_64bit_per_row = u16::try_from((hi >> 9) & 0x1FF).unwrap();
    let tmem_texture_offset = u16::try_from((hi >> 0) & 0x1FF).unwrap();
    let tile_descriptor_index = u8::try_from((lo >> 24) & 0b111).unwrap();
    let palette_index = u8::try_from((lo >> 20) & 0b1111).unwrap();
    let clamp_t = ((lo >> 19) & 0b1) != 0;
    let mirror_t = ((lo >> 18) & 0b1) != 0;
    let mask_t = u8::try_from((lo >> 14) & 0b1111).unwrap();
    let shift_t = u8::try_from((lo >> 10) & 0b1111).unwrap();
    let clamp_s = ((lo >> 9) & 0b1) != 0;
    let mirror_s = ((lo >> 8) & 0b1) != 0;
    let mask_s = u8::try_from((lo >> 4) & 0b1111).unwrap();
    let shift_s = u8::try_from((lo >> 0) & 0b1111).unwrap();

    let color_format = ColorFormat::from_base_type(color_format_u8);
    let bits_per_pixel = BitsPerPixel::from_base_type(bits_per_pixel_u8);

    Ok(GraphicsCommand::SetTile {
        color_format,
        bits_per_pixel,
        values_64bit_per_row,
        tmem_texture_offset,
        tile_descriptor_index,
        palette_index,
        clamp_t,
        mirror_t,
        mask_t,
        shift_t,
        clamp_s,
        mirror_s,
        mask_s,
        shift_s,
    })
}

pub fn decode_set_tile_size(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO SSSS SSSS SSSS TTTT TTTT TTTT
    // lo: xxxx xIII UUUU UUUU UUUU VVVV VVVV VVVV
    // O = opcode, I = tile index,
    // S = upper-left corner of S, T = upper-left corner of T,
    // U = lower-right corner of S, V = lower-right corner of T
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xF2);
    let tile_descriptor_index = u8::try_from((lo >> 24) & 0b111).unwrap();
    let upper_left_s = u16::try_from((hi >> 12) & 0x0FFF).unwrap();
    let upper_left_t = u16::try_from((hi >>  0) & 0x0FFF).unwrap();
    let lower_right_s = u16::try_from((lo >> 12) & 0x0FFF).unwrap();
    let lower_right_t = u16::try_from((lo >>  0) & 0x0FFF).unwrap();

    Ok(GraphicsCommand::SetTileSize {
        tile_descriptor_index,
        upper_left_s,
        upper_left_t,
        lower_right_s,
        lower_right_t,
    })
}

pub fn decode_pre2_set_geometry_mode(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: MMMM MMMM MMMM MMMM MMMM MMMM MMMM MMMM
    // O = opcode, M = mode bits to set
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xB7);
    Ok(GraphicsCommand::GeometryMode {
        set_bits: lo,
        clear_bits: 0,
    })
}

pub fn decode_pre2_clear_geometry_mode(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: MMMM MMMM MMMM MMMM MMMM MMMM MMMM MMMM
    // O = opcode, M = mode bits to clear
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xB6);
    Ok(GraphicsCommand::GeometryMode {
        set_bits: 0,
        clear_bits: lo,
    })
}

pub fn decode_pre2_set_other_mode(upper_half: bool, hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx SSSS SSSS LLLL LLLL
    // lo: DDDD DDDD DDDD DDDD DDDD DDDD DDDD DDDD
    // O = opcode, S = shift, L = length, D = data
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let expected_opcode = if upper_half { 0xBA } else { 0xB9 };
    assert_eq!(opcode, expected_opcode);
    let shift = u8::try_from((hi >> 8) & 0xFF).unwrap();
    let length = u8::try_from((hi >> 0) & 0xFF).unwrap();
    let data = lo;
    Ok(GraphicsCommand::SetOtherMode {
        upper_half,
        shift,
        length,
        data,
    })
}

pub fn decode_pre2_texture(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxLL LNNN AAAA AAAA
    // lo: SSSS SSSS SSSS SSSS TTTT TTTT TTTT TTTT
    // O = opcode, L = max mipmap level, N = tile number, A = on (active) flag,
    // S and T = texture coordinate scaling (fractional part)

    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xBB);
    let max_mipmap_levels = u8::try_from((hi >> 11) & 0b111).unwrap();
    let tile_number = u8::try_from((hi >> 8) & 0b111).unwrap();
    let on = ((hi >> 0) & 0xFF) != 0;
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

pub fn decode_load_block(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO SSSS SSSS SSSS TTTT TTTT TTTT
    // lo: xxxx xNNN LLLL LLLL LLLL CCCC CCCC CCCC
    // O = opcode, S = upper left S, T = upper left T, N = tile number,
    // L = lower right S, C = change of T per scan line

    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    assert_eq!(opcode, 0xF3);
    let upper_left_s = u16::try_from((hi >> 12) & 0x0FFF).unwrap();
    let upper_left_t = u16::try_from((hi >>  0) & 0x0FFF).unwrap();
    let tile_number = u8::try_from((lo >> 24) & 0b111).unwrap();
    let lower_right_s = u16::try_from((lo >> 12) & 0x0FFF).unwrap();
    let t_change_per_scan_line = u16::try_from((lo >> 0) & 0x0FFF).unwrap();
    Ok(GraphicsCommand::LoadBlock {
        tile_number,
        upper_left_s,
        upper_left_t,
        lower_right_s,
        t_change_per_scan_line,
    })
}

pub fn decode_rdp_load_sync(hi: u32, lo: u32) -> Result<GraphicsCommand, Error> {
    // hi: OOOO OOOO xxxx xxxx xxxx xxxx xxxx xxxx
    // lo: xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
    // O = opcode
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let _ = lo;
    assert_eq!(opcode, 0xE6);
    Ok(GraphicsCommand::RdpLoadSync)
}

fn util_decode_set_texture_image_opcode_format_bpp_width(hi: u32) -> Result<(u8, ColorFormat, BitsPerPixel, u16), Error> {
    // hi: OOOO OOOO FFFS Sxxx xxxx WWWW WWWW WWWW
    // O = opcode, F = format, S = size, W = width
    let opcode = u8::try_from((hi >> 24) & 0xFF).unwrap();
    let color_format_u8 = u8::try_from((hi >> 21) & 0b111).unwrap();
    let bits_per_pixel_u8 = u8::try_from((hi >> 19) & 0b11).unwrap();
    let width_value = u16::try_from((hi >> 0) & 0x0FFF).unwrap();

    let color_format = ColorFormat::from_base_type(color_format_u8);
    let bits_per_pixel = BitsPerPixel::from_base_type(bits_per_pixel_u8);
    let width = width_value + 1;

    Ok((opcode, color_format, bits_per_pixel, width))
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
