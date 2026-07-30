use crate::{brunsli::jpeg_data::{JPEGComponent, JPEGData, JPEGReadError, K_DCT_BLOCK_SIZE, K_MAX_DIM_PIXELS}, common::constants::{K_BRUNSLI_MAX_NUM_BLOCKS, K_BRUNSLI_MAX_SAMPLING}, enc::jpeg_data_reader::JPEGReadMode::JpegReadAll};
use std::cmp::max;
pub(crate) enum JPEGReadMode {
  JpegReadHeader,   // only basic headers
  JpegReadTables,   // headers and tables (quant, Huffman, ...)
  JpegReadAll,      // everything
}

#[inline(always)]
fn brunsli_verify_len(pos: usize, len: usize, n: usize, jpg: &mut JPEGData) -> bool {
    if pos + n > len {
        eprintln!(
            "Unexpected end of input: pos={} need={} len={}", 
            pos, 
            n, 
            len
        );
        jpg.error = JPEGReadError::UnexpectedEof;
        return false;
    }
    true
}

#[inline(always)]
fn brunsli_verify_input(var: i32, low: i32, high: i32, code: JPEGReadError, jpg: &mut JPEGData) -> bool {
    if var < low || var > high {
        eprintln!(
            "Invalid {}: {}", 
            stringify!(var), 
            var
        );
        jpg.error = code;
        return false;
    }
    true
}

#[inline(always)]
fn brunsli_verify_marker_end(start_pos: usize, pos: usize, marker_len: usize, jpg: &mut JPEGData) -> bool {
    if start_pos + marker_len != pos {
        eprintln!(
            "Invalid marker length: declared={} actual={}", 
            marker_len, 
            pos - start_pos
        );
        jpg.error = JPEGReadError::WrongMarkerSize;
        return false;
    }
    true
}

#[inline(always)]
fn brunsli_expect_marker(pos: usize, len: usize, data: &[u8], jpg: &mut JPEGData) -> bool {
    if pos + 2 > len || data[pos] != 0xff {
        eprintln!(
            "Marker byte (0xff) expected, found: {} pos={} len={}",
            if pos < len { data[pos] } else { 0 }, 
            pos, 
            len
        );
        jpg.error = JPEGReadError::MarkerByteNotFound;
        return false;
    }
    true
}

#[inline]
fn div_ceil(a: i32, b: i32) -> i32 {
    (a + b - 1) / b
}

#[inline]
fn read_uint8(data: &[u8], pos: &mut usize) -> i32 {
    let ret = data[*pos];
    *pos += 1;
    ret as i32
}

#[inline]
fn read_uint16(data: &[u8], pos: &mut usize) -> i32 {
    let v: i32 = (data[*pos] as i32) << 8 + data[*pos+1] as i32;
    *pos += 2;
    v
}

pub(crate) fn process_sof(data: &[u8], mode: JPEGReadMode, pos: &mut usize, jpg: &mut JPEGData) -> bool {
    if jpg.width != 0 {
        eprintln!("Duplicate SOF marker.");
        jpg.error = JPEGReadError::DuplicateSof;
        return false;
    }
    let start_pos: usize = *pos;
    if !brunsli_verify_len(*pos, data.len(), 8, jpg) {
        return false;
    }

    let marker_len: usize = read_uint16(data, pos) as usize;
    let precision: i32 = read_uint8(data, pos);
    let height: i32 = read_uint16(data, pos);
    let width: i32 = read_uint16(data, pos);
    let num_components: i32 = read_uint8(data, pos);
    if !brunsli_verify_input(precision, 8, 8, JPEGReadError::InvalidPrecision, jpg) {
        return false;
    }
    if !brunsli_verify_input(height, 1, K_MAX_DIM_PIXELS as i32, JPEGReadError::InvalidHeight, jpg) {
        return false;
    }
    if !brunsli_verify_input(width, 1, K_MAX_DIM_PIXELS as i32, JPEGReadError::InvalidWidth, jpg) {
        return false;
    }
    if !brunsli_verify_len(*pos, data.len(), 3 * num_components as usize, jpg) {
        return false;
    }
    jpg.height = height;
    jpg.width = width;
    jpg.components.resize(num_components as usize, JPEGComponent::default());

    let mut ids_seen: Vec<bool> = vec![false; 256];
    for i in 0..jpg.components.len() {
        let id: i32 = read_uint8(data, pos);
        if ids_seen[id as usize] {
            eprintln!(
                "Duplicated id {} in SOF.",
                id
            );
            jpg.error = JPEGReadError::DuplicateComponentId;
            return false;
        }
        ids_seen[id as usize] = true;
        jpg.components[i].id = id;
        let factor: i32 = read_uint8(data,pos);
        let h_samp_factor: i32 = factor >> 4;
        let v_samp_factor: i32 = factor & 0xf;
        if !brunsli_verify_input(h_samp_factor, 1, K_BRUNSLI_MAX_SAMPLING, JPEGReadError::InvalidSampFactor, jpg) {
            return false;
        }
        if !brunsli_verify_input(v_samp_factor, 1, K_BRUNSLI_MAX_SAMPLING, JPEGReadError::InvalidSampFactor, jpg) {
            return false;
        }
        jpg.components[i].h_samp_factor = h_samp_factor;
        jpg.components[i].v_samp_factor = v_samp_factor;
        jpg.components[i].quant_idx = read_uint8(data, pos) as u8;
        jpg.max_h_samp_factor = max(jpg.max_h_samp_factor, h_samp_factor);
        jpg.max_v_samp_factor = max(jpg.max_v_samp_factor, v_samp_factor);
    } 

    jpg.mcu_rows = div_ceil(jpg.height, jpg.max_v_samp_factor * 8);
    jpg.mcu_cols = div_ceil(jpg.width, jpg.max_h_samp_factor * 8);

    for i in 0..jpg.components.len() {
        let c: &mut JPEGComponent = &mut jpg.components[i];
        if jpg.max_h_samp_factor % c.h_samp_factor != 0
        || jpg.max_v_samp_factor % c.v_samp_factor != 0 {
            eprintln!("Non-integral subsampling ratios.");
            jpg.error = JPEGReadError::InvalidSamplingFactors;
            return false;
        }

        c.width_in_blocks = jpg.mcu_cols as u32 * c.h_samp_factor as u32;
        c.height_in_blocks = jpg.mcu_rows as u32 * c.v_samp_factor as u32;

        let num_blocks: u64 = u64::from(c.width_in_blocks) * u64::from(c.height_in_blocks);
        if num_blocks > K_BRUNSLI_MAX_NUM_BLOCKS as u64 {
            eprintln!("Image too large.");
            jpg.error = JPEGReadError::ImageTooLarge;
            return false;
        }
        c.num_blocks = num_blocks as u32;
        if matches!(mode, JPEGReadMode::JpegReadAll) {
            c.coeffs.resize(c.num_blocks as usize * K_DCT_BLOCK_SIZE, 0);
        }
    }
    if !brunsli_verify_marker_end(start_pos, *pos, marker_len, jpg) {
        return false;
    }
    true
}