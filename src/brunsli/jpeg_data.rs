pub(crate) const K_FALLBACK_VERSION : i32 = 1;

pub(crate) const K_DCT_BLOCK_SIZE : usize = 64;
pub(crate) const K_MAX_COMPONENTS : usize = 4;
pub(crate) const K_MAX_QUANT_TABLES : usize = 4;
pub(crate) const K_MAX_HUFFMAN_TABLES : usize = 4;
pub(crate) const K_JPEG_HUFFMAN_MAX_BIT_LENGTH : usize = 16;
pub(crate) const K_JPEG_HUFFMAN_ALPHABET_SIZE : usize = 256;
pub(crate) const K_JPEG_DC_ALPHABET_SIZE : usize = 12;
pub(crate) const K_MAX_DHT_MARKERS : usize = 512;
pub(crate) const K_MAX_DIM_PIXELS : usize = 65535;

pub(crate) const K_DEFAULT_QUANT_MATRIX: [[u8; 64]; 2] = [
    [16,  11,  10,  16,  24,  40,  51,  61,
    12,  12,  14,  19,  26,  58,  60,  55,
    14,  13,  16,  24,  40,  57,  69,  56,
    14,  17,  22,  29,  51,  87,  80,  62,
    18,  22,  37,  56,  68, 109, 103,  77,
    24,  35,  55,  64,  81, 104, 113,  92,
    49,  64,  78,  87, 103, 121, 120, 101,
    72,  92,  95,  98, 112, 100, 103,  99],
    [17,  18,  24,  47,  99,  99,  99,  99,
    18,  21,  26,  66,  99,  99,  99,  99,
    24,  26,  56,  99,  99,  99,  99,  99,
    47,  66,  99,  99,  99,  99,  99,  99,
    99,  99,  99,  99,  99,  99,  99,  99,
    99,  99,  99,  99,  99,  99,  99,  99,
    99,  99,  99,  99,  99,  99,  99,  99,
    99,  99,  99,  99,  99,  99,  99,  99]
];

pub(crate) const K_JPEG_NATURAL_ORDER: [u32; 80] = [
  0,   1,  8, 16,  9,  2,  3, 10,
  17, 24, 32, 25, 18, 11,  4,  5,
  12, 19, 26, 33, 40, 48, 41, 34,
  27, 20, 13,  6,  7, 14, 21, 28,
  35, 42, 49, 56, 57, 50, 43, 36,
  29, 22, 15, 23, 30, 37, 44, 51,
  58, 59, 52, 45, 38, 31, 39, 46,
  53, 60, 61, 54, 47, 55, 62, 63,
  // extra entries for safety in decoder
  63, 63, 63, 63, 63, 63, 63, 63,
  63, 63, 63, 63, 63, 63, 63, 63
];

pub(crate) const K_JPEG_ZIG_ZAG_ORDER: [u32; 64] = [
  0,   1,  5,  6, 14, 15, 27, 28,
  2,   4,  7, 13, 16, 26, 29, 42,
  3,   8, 12, 17, 25, 30, 41, 43,
  9,  11, 18, 24, 31, 40, 44, 53,
  10, 19, 23, 32, 39, 45, 52, 54,
  20, 22, 33, 38, 46, 51, 55, 60,
  21, 34, 37, 47, 50, 56, 59, 61,
  35, 36, 48, 49, 57, 58, 62, 63
];

pub(crate) enum JPEGReadError {
  Ok,
  SoiNotFound,
  SofNotFound,
  UnexpectedEof,
  MarkerByteNotFound,
  UnsupportedMarker,
  WrongMarkerSize,
  InvalidPrecision,
  InvalidWidth,
  InvalidHeight,
  InvalidNumComp,
  InvalidSampFactor,
  InvalidStartOfScan,
  InvalidEndOfScan,
  InvalidScanBitPosition,
  InvalidCompsInScan,
  InvalidHuffmanIndex,
  InvalidQuantTblIndex,
  InvalidQuantVal,
  InvalidMarkerLen,
  InvalidSamplingFactors,
  InvalidHuffmanCode,
  InvalidSymbol,
  NonRepresentableDcCoeff,
  NonRepresentableAcCoeff,
  InvalidScan,
  OverlappingScans,
  InvalidScanOrder,
  ExtraZeroRun,
  DuplicateDri,
  DuplicateSof,
  WrongRestartMarker,
  DuplicateComponentId,
  ComponentNotFound,
  HuffmanTableNotFound,
  HuffmanTableError,
  QuantTableNotFound,
  EmptyDht,
  EmptyDqt,
  OutOfBandCoeff,
  EobRunTooLong,
  ImageTooLarge,
  InvalidQuantTblPrecision,
}

pub(crate) struct JPEGQuantTable {
    pub(crate) values : [i32; K_DCT_BLOCK_SIZE],
    pub(crate) precision : i32,
    pub(crate) index : i32,
    pub(crate) is_last : bool,
}

impl Default for JPEGQuantTable {
    fn default() -> Self {
        Self {
            values : [0; K_DCT_BLOCK_SIZE],
            precision : 0,
            index : 0,
            is_last : true,
        }
    }
}

pub(crate) struct JPEGHuffmanCode {
    pub(crate) counts : [i32; K_JPEG_HUFFMAN_MAX_BIT_LENGTH + 1],
    pub(crate) values : [i32; K_JPEG_HUFFMAN_ALPHABET_SIZE + 1],
    pub(crate) slot_id : i32,
    pub(crate) is_last : bool,
}

impl Default for JPEGHuffmanCode {
    fn default() -> Self {
        Self {
            counts: [0; K_JPEG_HUFFMAN_MAX_BIT_LENGTH + 1],
            values: [0; K_JPEG_HUFFMAN_ALPHABET_SIZE + 1],
            slot_id: 0,
            is_last: true,
        }
    }
}

#[derive(Default)]
pub(crate) struct JPEGComponentScanInfo {
    pub(crate) comp_idx : u8,
    pub(crate) dc_tbl_idx : i32,
    pub(crate) ac_tbl_idx : i32,
}

#[derive(Default)]
pub(crate) struct ExtraZeroRunInfo {
    pub(crate) block_idx : i32,
    pub(crate) num_extra_zero_runs : i32,
}

#[derive(Default)]
pub(crate) struct JPEGScanInfo {
    pub(crate) ss : i32,
    pub(crate) se : i32,
    pub(crate) ah : i32,
    pub(crate) al : i32,
    pub(crate) num_components : usize,
    pub(crate) components : [JPEGComponentScanInfo; 4],
    pub(crate) reset_points : Vec<i32>,
    pub(crate) extra_zero_runs : Vec<ExtraZeroRunInfo>,
}

type Coeff = i16;

#[derive(Clone)]
pub(crate) struct JPEGComponent {
    pub(crate) id : i32,
    pub(crate) h_samp_factor : i32,
    pub(crate) v_samp_factor : i32,
    pub(crate) quant_idx : u8,
    pub(crate) width_in_blocks : u32,
    pub(crate) height_in_blocks : u32,
    pub(crate) num_blocks : u32,
    pub(crate) coeffs : Vec<Coeff>,
}

impl Default for JPEGComponent {
    fn default() -> Self {
        Self {
            id: 0,
            h_samp_factor: 1,
            v_samp_factor: 1,
            quant_idx: 0,
            width_in_blocks: 0,
            height_in_blocks: 0,
            num_blocks: 0,
            coeffs: Vec::new(),
        }
    }
}

pub(crate) struct JPEGData {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) version: i32,
    pub(crate) max_h_samp_factor: i32,
    pub(crate) max_v_samp_factor: i32,
    pub(crate) mcu_rows: i32,
    pub(crate) mcu_cols: i32,
    pub(crate) restart_interval: i32,
    pub(crate) app_data: Vec<Vec<u8>>,
    pub(crate) com_data: Vec<Vec<u8>>,
    pub(crate) quant: Vec<JPEGQuantTable>,
    pub(crate) huffman_code: Vec<JPEGHuffmanCode>,
    pub(crate) components: Vec<JPEGComponent>,
    pub(crate) scan_info: Vec<JPEGScanInfo>,
    pub(crate) marker_order: Vec<u8>,
    pub(crate) inter_marker_data: Vec<Vec<u8>>,
    pub(crate) tail_data: Vec<u8>,
    pub(crate) original_jpg: Option<Vec<u8>>,
    pub(crate) error: JPEGReadError,
    pub(crate) has_zero_padding_bit: bool,
    pub(crate) padding_bits: Vec<i32>,
}

impl Default for JPEGData {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            version: 2,
            max_h_samp_factor: 1,
            max_v_samp_factor: 1,
            mcu_rows: 0,
            mcu_cols: 0,
            restart_interval: 0,
            app_data: Vec::new(),
            com_data: Vec::new(),
            quant: Vec::new(),
            huffman_code: Vec::new(),
            components: Vec::new(),
            scan_info: Vec::new(),
            marker_order: Vec::new(),
            inter_marker_data: Vec::new(),
            tail_data: Vec::new(),
            original_jpg: None,
            error: JPEGReadError::Ok,
            has_zero_padding_bit: false,
            padding_bits: Vec::new(),
        }
    }
}

impl JPEGData {
    #[inline]
    fn jpegdata_is420(&self) -> bool {
        self.components.len() == 3 && 
        self.max_h_samp_factor == 2 &&
        self.max_v_samp_factor == 2 && 
        self.components[0].h_samp_factor == 2 &&
        self.components[0].v_samp_factor == 2 &&
        self.components[1].h_samp_factor == 1 &&
        self.components[1].v_samp_factor == 1 &&
        self.components[2].h_samp_factor == 1 &&
        self.components[2].v_samp_factor == 1
    }

    #[inline]
    fn jpegdata_is444(&self) -> bool {
        self.components.len() == 3 && 
        self.max_h_samp_factor == 1 &&
        self.max_v_samp_factor == 1 && 
        self.components[0].h_samp_factor == 1 &&
        self.components[0].v_samp_factor == 1 &&
        self.components[1].h_samp_factor == 1 &&
        self.components[1].v_samp_factor == 1 &&
        self.components[2].h_samp_factor == 1 &&
        self.components[2].v_samp_factor == 1       
    }

    #[inline]
    fn padding_bits_limit(&self) -> u64 {
        let num_blocks : u64 = ((self.width as u64 + 15) >> 3) * ((self.height as u64 + 15) >> 3);
        7u64 * num_blocks * (self.components.len() as u64) + 256u64 
    }
}