# Golden reference tests

This directory holds small standalone C++ programs that compile directly
against the real upstream brunsli C++ source (`brunsli/` submodule) and dump
the output of a function for a handful of hand-picked inputs.

These are **not** run automatically and are not part of `cargo test` — they
are one-time reference generators. The workflow is:

1. Write a harness `.cc` file here that `#include`s the real C++ header/impl
   you're translating and calls it with the same inputs you're about to write
   a Rust test for.
2. Compile and run it (see per-file build commands, or the generic one
   below).
3. Copy its printed output into the corresponding Rust `#[cfg(test)] mod
   tests` block as golden/expected values, so the Rust test asserts
   byte-for-byte agreement with the reference implementation rather than
   hand-derived expectations.
4. Re-run the harness if you add more cases later, to keep cross-checking
   against the reference.

## Building

Compiled binaries go in `golden/build/`, which is gitignored — only the
`.cc` sources are checked in.

```
g++ -std=c++17 -I. -Ibrunsli/c/include -o golden/build/<name> golden/<name>.cc <any .cc files it depends on>
./golden/build/<name>
```

## Files

- `jpeg_huffman_decode_golden.cc` — reference for
  `enc::jpeg_huffman_decode::build_jpeg_huffman_table`, cross-checked against
  `brunsli/c/enc/jpeg_huffman_decode.cc`'s `BuildJpegHuffmanTable`.