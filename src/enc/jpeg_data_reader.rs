use crate::brunsli::jpeg_data::{JPEGReadError, JPEGData};

#[inline(always)]
fn brunsli_verify_len(pos: usize, len: usize, n: usize, jpg: &mut JPEGData) -> bool {
    if pos + n > len {
        eprintln!("Unexpected end of input: pos={} need={} len={}", pos, n, len);
        jpg.error = JPEGReadError::UnexpectedEof;
        return false;
    }
    true
}

