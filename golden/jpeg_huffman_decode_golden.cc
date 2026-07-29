// Golden-reference harness for enc::jpeg_huffman_decode::build_jpeg_huffman_table.
//
// Compiles directly against the real upstream C++ BuildJpegHuffmanTable
// (brunsli/c/enc/jpeg_huffman_decode.cc) and dumps its output for a handful
// of hand-picked inputs. The printed values were used to write the golden
// assertions in src/enc/jpeg_huffman_decode.rs's #[cfg(test)] module, so this
// file is not run automatically (no ongoing C++ build dependency) — it's a
// one-time reference generator. Re-run it if you add more cases to
// cross-check against the reference implementation.
//
// Build & run from the repo root:
//   g++ -std=c++17 -I. -Ibrunsli/c/include -o golden/build/jpeg_huffman_decode_golden \
//       golden/jpeg_huffman_decode_golden.cc brunsli/c/enc/jpeg_huffman_decode.cc
//   golden/build/jpeg_huffman_decode_golden

#include <cstdio>
#include <vector>
#include "brunsli/c/enc/jpeg_huffman_decode.h"

using namespace brunsli;

void dump(const char* name, const std::vector<int>& counts,
          const std::vector<int>& symbols, int print_first_n) {
  std::vector<HuffmanTableEntry> lut(kJpegHuffmanLutSize);
  BuildJpegHuffmanTable(counts.data(), symbols.data(), lut.data());

  printf("// --- %s ---\n", name);
  for (int i = 0; i < print_first_n; ++i) {
    printf("(%d, bits=%d, value=%d)\n", i, lut[i].bits, lut[i].value);
  }
  printf("\n");
}

int main() {
  // Single-symbol special case: every root entry should point at the same
  // symbol with bits == 0.
  {
    std::vector<int> counts(17, 0);
    counts[1] = 1;
    std::vector<int> symbols = {42};
    dump("single_symbol", counts, symbols, 256);
  }

  // Complete 2-bit code, 4 symbols: root table filled entirely by direct
  // codes, no 2nd-level tables needed.
  {
    std::vector<int> counts(17, 0);
    counts[2] = 4;
    std::vector<int> symbols = {10, 11, 12, 13};
    dump("complete_two_bit", counts, symbols, 256);
  }

  // Mixed lengths: 2 symbols of length 3, 6 of length 10. Short enough that
  // most of the root table needs 2nd-level pointer redirects, exercising the
  // root/2nd-level split and the sub-table-boundary logic.
  {
    std::vector<int> counts(17, 0);
    counts[3] = 2;
    counts[10] = 6;
    std::vector<int> symbols = {1, 2, 3, 4, 5, 6, 7, 8};
    dump("mixed_with_2nd_level", counts, symbols, 520);
  }

  return 0;
}