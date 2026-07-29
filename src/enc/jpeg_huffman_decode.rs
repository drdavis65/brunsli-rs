use crate::brunsli::jpeg_data::{K_JPEG_HUFFMAN_MAX_BIT_LENGTH};

const K_JPEG_HUFFMAN_ROOT_TABLE_BITS : i32 = 8;
// Maximum huffman lookup table size.
// Requirements: alphabet of 257 symbols (256 + 1 special symbol for the all 1s
// code) and max bit length 16, the root table has 8 bits.
// zlib/examples/enough.c works with an assumption that Huffman code is
// "complete". Input JPEGs might have this assumption broken, hence the
// following sum is used as estimate:
//  + number of 1-st level cells
//  + number of symbols
//  + assymptotic amount of repeated 2-nd level cells
// The third number is 1 + 3 + ... + 255 i.e. it is assumed that sub-table of
// each "size" might be almost completely be filled with repetitions.
// Total sum is slightly less than 1024,...
const K_JPEG_HUFFMAN_LUT_SIZE : i32 = 1024;

#[derive(Clone, Copy)]
pub(crate) struct HuffmanTableEntry {
  // Initialize the value to an invalid symbol so that we can recognize it
  // when reading the bit stream using a Huffman code with space > 0.
//  HuffmanTableEntry() : bits(0), value(0xffff) {}

  bits: u8,     // number of bits used for this symbol
  value : u16,   // symbol value or table offset
}

impl Default for HuffmanTableEntry {
    fn default() -> Self {
        Self {
            bits: 0,
            value: 0xffff,
        }
    }
}

#[inline]
fn next_table_bit_size(count: &[i32], mut len: i32) -> i32 {
    let mut left: i32 = 1 << (len - K_JPEG_HUFFMAN_ROOT_TABLE_BITS);
    while len < K_JPEG_HUFFMAN_MAX_BIT_LENGTH as i32 {
        left -= count[len as usize];
        if left <= 0 {
            break;
        }
        len += 1;
        left <<= 1;
    }
    return len - K_JPEG_HUFFMAN_ROOT_TABLE_BITS
}

// Builds jpeg-style Huffman lookup table from the given symbols.
// The symbols are in order of increasing bit lengths. The number of symbols
// with bit length n is given in counts[n] for each n >= 1.
pub(crate) fn build_jpeg_huffman_table(count: &[i32], symbols: &[i32], lut: &mut [HuffmanTableEntry]) -> () {
    let mut code = HuffmanTableEntry { bits: 0, value: 0};
    //let table: &mut [HuffmanTableEntry];
    let mut idx: usize;
    let mut key: usize;
    let mut reps: usize;
    let mut low: i32;
    let mut table_bits: i32;
    let mut table_size: usize;
    
    let mut tmp_count: [i32; K_JPEG_HUFFMAN_MAX_BIT_LENGTH + 1] = [0; K_JPEG_HUFFMAN_MAX_BIT_LENGTH + 1];
    tmp_count.copy_from_slice(count);
    tmp_count[0] = 0;
    let total_count: i32;

    total_count = tmp_count.iter().sum();

    //table = lut;
    table_bits = K_JPEG_HUFFMAN_ROOT_TABLE_BITS;
    table_size = 1 << table_bits;

    if total_count == 1 {
        code.value = symbols[0] as u16;
        lut[..table_size].fill(code);
        return;
    }

    key = 0;
    idx = 0;

    for len in 1..=K_JPEG_HUFFMAN_ROOT_TABLE_BITS as usize{
        while tmp_count[len] > 0 {
            code.bits = len as u8;
            code.value = symbols[idx] as u16;
            idx += 1;
            reps = 1 << (K_JPEG_HUFFMAN_ROOT_TABLE_BITS as usize - len);
            lut[key..key + reps].fill(code);
            key += reps;
            tmp_count[len] -= 1;
        }
    }

    let (root,mut remaining) = lut.split_at_mut(table_size);
    let mut remaining_base: usize = table_size;
    table_size = 0;
    low = 0;
    for len in K_JPEG_HUFFMAN_ROOT_TABLE_BITS as usize + 1..=K_JPEG_HUFFMAN_MAX_BIT_LENGTH {
        while tmp_count[len] > 0 {
            if low >= table_size as i32 {
                remaining = &mut remaining[table_size..];
                remaining_base += table_size;
                table_bits = next_table_bit_size(&tmp_count, len as i32);
                table_size = 1 << table_bits;
                low = 0;
                root[key].bits = table_bits as u8 + K_JPEG_HUFFMAN_ROOT_TABLE_BITS as u8;
                root[key].value = (remaining_base - key) as u16;
                key += 1;
            }
            code.bits = len as u8 - K_JPEG_HUFFMAN_ROOT_TABLE_BITS as u8;
            code.value = symbols[idx] as u16;
            idx += 1;
            reps = 1 << (table_bits - code.bits as i32);
            remaining[low as usize..low as usize + reps].fill(code);
            low += reps as i32;
            tmp_count[len] -= 1;
        }
    }
}

// Expected values below were captured by compiling and running the real
// upstream C++ `BuildJpegHuffmanTable` (brunsli/c/enc/jpeg_huffman_decode.cc)
// against the same inputs, so these assert byte-for-byte agreement with the
// reference implementation, not just internally-derived expectations.
#[cfg(test)]
mod tests {
    use super::*;

    fn counts_with(entries: &[(usize, i32)]) -> [i32; K_JPEG_HUFFMAN_MAX_BIT_LENGTH + 1] {
        let mut counts = [0i32; K_JPEG_HUFFMAN_MAX_BIT_LENGTH + 1];
        for &(len, n) in entries {
            counts[len] = n;
        }
        counts
    }

    #[test]
    fn single_symbol_fills_entire_root_table() {
        let counts = counts_with(&[(1, 1)]);
        let symbols = [42i32];
        let mut lut = [HuffmanTableEntry::default(); K_JPEG_HUFFMAN_LUT_SIZE as usize];

        build_jpeg_huffman_table(&counts, &symbols, &mut lut);

        for entry in &lut[..256] {
            assert_eq!(entry.bits, 0);
            assert_eq!(entry.value, 42);
        }
    }

    #[test]
    fn complete_two_bit_code_fills_root_table_with_repeats() {
        let counts = counts_with(&[(2, 4)]);
        let symbols = [10i32, 11, 12, 13];
        let mut lut = [HuffmanTableEntry::default(); K_JPEG_HUFFMAN_LUT_SIZE as usize];

        build_jpeg_huffman_table(&counts, &symbols, &mut lut);

        for entry in &lut[0..64] {
            assert_eq!((entry.bits, entry.value), (2, 10));
        }
        for entry in &lut[64..128] {
            assert_eq!((entry.bits, entry.value), (2, 11));
        }
        for entry in &lut[128..192] {
            assert_eq!((entry.bits, entry.value), (2, 12));
        }
        for entry in &lut[192..256] {
            assert_eq!((entry.bits, entry.value), (2, 13));
        }
    }

    #[test]
    fn mixed_lengths_exercise_second_level_tables() {
        // Two 3-bit codes plus six 10-bit codes: short enough to leave most
        // of the root table needing 2nd-level pointer redirects.
        let counts = counts_with(&[(3, 2), (10, 6)]);
        let symbols = [1i32, 2, 3, 4, 5, 6, 7, 8];
        let mut lut = [HuffmanTableEntry::default(); K_JPEG_HUFFMAN_LUT_SIZE as usize];

        build_jpeg_huffman_table(&counts, &symbols, &mut lut);

        // Root table: two direct 3-bit codes, each repeated 32 times.
        for entry in &lut[0..32] {
            assert_eq!((entry.bits, entry.value), (3, 1));
        }
        for entry in &lut[32..64] {
            assert_eq!((entry.bits, entry.value), (3, 2));
        }
        // Root pointer-redirect entries into the two 2nd-level sub-tables.
        assert_eq!((lut[64].bits, lut[64].value), (10, 192));
        assert_eq!((lut[65].bits, lut[65].value), (16, 195));
        // Remaining root slots were never written (still default).
        for entry in &lut[66..256] {
            assert_eq!((entry.bits, entry.value), (0, 0xffff));
        }

        // First (4-entry) sub-table: one entry per symbol.
        assert_eq!((lut[256].bits, lut[256].value), (2, 3));
        assert_eq!((lut[257].bits, lut[257].value), (2, 4));
        assert_eq!((lut[258].bits, lut[258].value), (2, 5));
        assert_eq!((lut[259].bits, lut[259].value), (2, 6));

        // Second (256-entry) sub-table: remaining two symbols, each repeated 64 times.
        for entry in &lut[260..324] {
            assert_eq!((entry.bits, entry.value), (2, 7));
        }
        for entry in &lut[324..388] {
            assert_eq!((entry.bits, entry.value), (2, 8));
        }
        // Rest of the second sub-table was never written (still default).
        for entry in &lut[388..516] {
            assert_eq!((entry.bits, entry.value), (0, 0xffff));
        }
    }
}


