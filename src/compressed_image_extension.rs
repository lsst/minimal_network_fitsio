use crate::compression::{GZip1, GZip2, NoCompress};
use crate::pixel_transform::{
    I8Transform, NoDitherQuantization, U16Transform, U32Transform, U64Transform,
};
use crate::tile_reader::{Address, BasicTileReader, TypedTileReader};
use crate::{Column, FitsIntegerType, Schema};

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum CompressionAlgorithm {
    // RICE_1 { blocksize: i8, bytepix: i8 }, [not implemented]
    GZIP_1,
    GZIP_2,
    // PLIO_1, [not implemented]
    // HCOMPRESS_1 { scale: f64, smooth: bool }, [not implemented]
    NOCOMPRESS,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuantizationAlgorithm {
    NO_DITHER,
    // SUBTRACTIVE_DITHER_1(u16), [not implemented]
    // SUBTRACTIVE_DITHER_2(u16), [not implemented]
}

#[derive(Clone, Debug)]
pub struct Quantization {
    pub algorithm: QuantizationAlgorithm,
    pub stored: FitsIntegerType,
    pub zero: f64,
    pub scale: f64,
    pub blank: Option<i64>,
}

#[derive(Clone, Debug)]
pub enum PixelType {
    U8,
    I8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32(Option<Quantization>),
    F64(Option<Quantization>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileIndex {
    flat: usize,
    nd: Vec<usize>,
}

impl TileIndex {
    pub fn flat(&self) -> usize {
        self.flat
    }
    pub fn nd(&self) -> &[usize] {
        &self.nd
    }
}

#[derive(Clone)]
pub struct CompressedImageExtension {
    schema: Schema,
    pixel_type: PixelType,
    full_shape: Vec<usize>,
    grid_shape: Vec<usize>,
    algorithm: CompressionAlgorithm,
}

impl CompressedImageExtension {
    pub fn from_schema(
        schema: Schema,
        pixel_type: PixelType,
        full_shape: Vec<usize>,
        tile_shape: Vec<usize>,
        algorithm: CompressionAlgorithm,
    ) -> Self {
        // TODO: check for consistency between schema and the rest of the spec.
        let grid_shape = full_shape
            .iter()
            .zip(tile_shape.iter())
            .map(|(&f, &t)| f.div_ceil(t))
            .collect();
        Self {
            schema,
            pixel_type,
            full_shape,
            grid_shape,
            algorithm,
        }
    }
    pub fn tile_reader(&self, index: &TileIndex, row_bytes: &[u8]) -> TypedTileReader {
        // If this is the last tile in one or more dimensions, shrink the size
        // in that dimension if the full shape and tile shape don't divide
        // cleanly.
        let shape = self.tile_shape(index);
        let (address, algorithm, quantization) = self.read_row(row_bytes);
        make_tile_reader(address, shape, &self.pixel_type, algorithm, quantization).unwrap()
    }
    pub fn index_from_flat(&self, index_flat: usize) -> TileIndex {
        let mut index_nd = vec![0; self.grid_shape.len()];
        let mut d = index_flat;
        for (i, n) in index_nd.iter_mut().zip(self.grid_shape.iter()).rev() {
            *i = d % n;
            d /= n;
        }
        TileIndex {
            flat: index_flat,
            nd: index_nd,
        }
    }
    pub fn index_from_nd(&self, index_nd: Vec<usize>) -> TileIndex {
        let mut index_flat = 0;
        for (i, n) in index_nd.iter().zip(self.grid_shape.iter()) {
            index_flat *= n;
            index_flat += i;
        }
        TileIndex {
            flat: index_flat,
            nd: index_nd,
        }
    }
    pub fn tile_shape(&self, index: &TileIndex) -> Vec<usize> {
        self.full_shape
            .iter()
            .zip(self.grid_shape.iter().zip(index.nd().iter()))
            .map(|(&n, (&g, &i))| {
                if n % g == 0 || i < g - 1 {
                    // grid divides full image size cleanly
                    n.div_ceil(g)
                } else {
                    n % g
                }
            })
            .collect()
    }

    fn read_row(&self, row_bytes: &[u8]) -> (Address, CompressionAlgorithm, Option<Quantization>) {
        let mut algorithm: CompressionAlgorithm = self.algorithm.clone();
        let mut address: Option<Address> = None;
        let mut quantization = match &self.pixel_type {
            PixelType::F32(quantization) => quantization.clone(),
            PixelType::F64(quantization) => quantization.clone(),
            _ => None,
        };
        for (column, row_offset) in self.schema.columns() {
            match column {
                Column::COMPRESSED_DATA(pointer_type, _) => {
                    assert!(address.is_none(), "Multiple data columns set.");
                    address = pointer_type.read_address(row_bytes, *row_offset);
                }
                Column::GZIP_COMPRESSED_DATA(pointer_type) => {
                    assert!(address.is_none(), "Multiple data columns set.");
                    address = pointer_type.read_address(row_bytes, *row_offset);
                    if address.is_some() {
                        algorithm = CompressionAlgorithm::GZIP_1;
                    }
                }
                Column::UNCOMPRESSED_DATA(pointer_type) => {
                    assert!(address.is_none(), "Multiple data columns set.");
                    address = pointer_type.read_address(row_bytes, *row_offset);
                    if address.is_some() {
                        algorithm = CompressionAlgorithm::NOCOMPRESS;
                    }
                }
                Column::ZZERO(col_type) => {
                    quantization
                        .as_mut()
                        .expect("ZZERO column with no quantization")
                        .zero = col_type.read(row_bytes, *row_offset);
                }
                Column::ZSCALE(col_type) => {
                    quantization
                        .as_mut()
                        .expect("SCALE column with no quantization")
                        .scale = col_type.read(row_bytes, *row_offset);
                }
                Column::ZBLANK(col_type) => {
                    quantization
                        .as_mut()
                        .expect("BLANK column with no quantization")
                        .blank = Some(col_type.read(row_bytes, *row_offset));
                }
                Column::Other(..) => {}
            }
        }
        (
            address.expect("No data column set."),
            algorithm,
            quantization,
        )
    }
}

fn make_tile_reader(
    address: Address,
    shape: Vec<usize>,
    pixel_type: &PixelType,
    algorithm: CompressionAlgorithm,
    quantization: Option<Quantization>,
) -> Option<TypedTileReader> {
    macro_rules! make_tile_reader {
        ($target: tt) => {
            match algorithm {
                CompressionAlgorithm::GZIP_1 => Some(TypedTileReader::$target(
                    BasicTileReader::new_boxed(address, shape, GZip1::new()),
                )),
                CompressionAlgorithm::GZIP_2 => Some(TypedTileReader::$target(
                    BasicTileReader::new_boxed(address, shape, GZip2::new()),
                )),
                CompressionAlgorithm::NOCOMPRESS => Some(TypedTileReader::$target(
                    BasicTileReader::new_boxed(address, shape, NoCompress::new()),
                )),
            }
        };
    }
    macro_rules! make_tile_reader_transformed {
        ($target: tt, $transform: expr) => {
            match algorithm {
                CompressionAlgorithm::GZIP_1 => Some(TypedTileReader::$target(
                    BasicTileReader::new(address, shape, GZip1::new()).transformed($transform),
                )),
                CompressionAlgorithm::GZIP_2 => Some(TypedTileReader::$target(
                    BasicTileReader::new(address, shape, GZip2::new()).transformed($transform),
                )),
                CompressionAlgorithm::NOCOMPRESS => Some(TypedTileReader::$target(
                    BasicTileReader::new(address, shape, NoCompress::new()).transformed($transform),
                )),
            }
        };
    }
    macro_rules! make_tile_reader_scaled {
        ($target: tt, $original: ty, $stored: ty, $q: expr) => {
            make_tile_reader_transformed!(
                $target,
                NoDitherQuantization::<$original, $stored>::new(
                    $q.zero as $original,
                    $q.scale as $original,
                    $q.blank.map(|b| b as $stored)
                )
            )
        };
    }
    macro_rules! make_tile_reader_float {
        ($target: tt, $original: ty, $q: expr) => {
            if let Some(q) = $q {
                match q.algorithm {
                    QuantizationAlgorithm::NO_DITHER => match q.stored {
                        FitsIntegerType::U8 => make_tile_reader_scaled!($target, $original, u8, q),
                        FitsIntegerType::I16 => {
                            make_tile_reader_scaled!($target, $original, i16, q)
                        }
                        FitsIntegerType::I32 => {
                            make_tile_reader_scaled!($target, $original, i32, q)
                        }
                        FitsIntegerType::I64 => {
                            make_tile_reader_scaled!($target, $original, i64, q)
                        }
                    },
                }
            } else {
                make_tile_reader!(F32)
            }
        };
    }
    match pixel_type {
        PixelType::U8 => make_tile_reader!(U8),
        PixelType::I8 => make_tile_reader_transformed!(I8, I8Transform()),
        PixelType::U16 => make_tile_reader_transformed!(U16, U16Transform()),
        PixelType::I16 => make_tile_reader!(I16),
        PixelType::U32 => make_tile_reader_transformed!(U32, U32Transform()),
        PixelType::I32 => make_tile_reader!(I32),
        PixelType::U64 => make_tile_reader_transformed!(U64, U64Transform()),
        PixelType::I64 => make_tile_reader!(I64),
        PixelType::F32(_) => make_tile_reader_float!(F32, f32, quantization),
        PixelType::F64(_) => make_tile_reader_float!(F64, f64, quantization),
    }
}
