use crate::tile_reader::Address;
use std::convert::Into;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerType {
    I32,
    I64,
}

impl PointerType {
    fn bytes(&self) -> usize {
        match self {
            Self::I32 => 4,
            Self::I64 => 8,
        }
    }
    pub(crate) fn read_address(&self, row: &[u8], offset: usize) -> Option<Address> {
        let address = match self {
            Self::I32 => Address {
                size: i32::from_be_bytes(row[offset..offset + 4].try_into().unwrap())
                    .try_into()
                    .expect("Negative value in dynamic array size."),
                offset: i32::from_be_bytes(row[offset + 4..offset + 8].try_into().unwrap())
                    .try_into()
                    .expect("Negative value in dynamic array offset."),
            },
            Self::I64 => Address {
                size: i64::from_be_bytes(row[offset..offset + 8].try_into().unwrap())
                    .try_into()
                    .expect("Negative value in dynamic array size."),
                offset: i64::from_be_bytes(row[offset + 8..offset + 16].try_into().unwrap())
                    .try_into()
                    .expect("Negative value in dynamic array offset."),
            },
        };
        if address.size != 0 {
            Some(address)
        } else {
            None
        }
    }
}

impl Into<char> for PointerType {
    fn into(self) -> char {
        match self {
            Self::I32 => 'P',
            Self::I64 => 'Q',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitsIntegerType {
    U8,
    I16,
    I32,
    I64,
}

impl FitsIntegerType {
    fn bytes(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
        }
    }
    pub(crate) fn read(&self, row: &[u8], offset: usize) -> i64 {
        match self {
            Self::U8 => row[offset].into(),
            Self::I16 => i16::from_be_bytes(row[offset..offset + 2].try_into().unwrap()).into(),
            Self::I32 => i32::from_be_bytes(row[offset..offset + 4].try_into().unwrap()).into(),
            Self::I64 => i64::from_be_bytes(row[offset..offset + 8].try_into().unwrap()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatType {
    F32,
    F64,
}

impl FloatType {
    pub fn bytes(&self) -> usize {
        match self {
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }
    pub(crate) fn read(&self, row: &[u8], offset: usize) -> f64 {
        match self {
            Self::F32 => f32::from_be_bytes(row[offset..offset + 4].try_into().unwrap()).into(),
            Self::F64 => f64::from_be_bytes(row[offset..offset + 8].try_into().unwrap()),
        }
    }
}

impl Into<char> for FloatType {
    fn into(self) -> char {
        match self {
            Self::F32 => 'E',
            Self::F64 => 'F',
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum Column {
    COMPRESSED_DATA(PointerType, FitsIntegerType),
    GZIP_COMPRESSED_DATA(PointerType),
    UNCOMPRESSED_DATA(PointerType),
    ZSCALE(FloatType),
    ZZERO(FloatType),
    ZBLANK(FitsIntegerType),
    Other(usize), // value is number of bytes this field occupies
}

impl Column {
    fn bytes(&self) -> usize {
        match self {
            Self::COMPRESSED_DATA(p, _) => p.bytes() * 2,
            Self::GZIP_COMPRESSED_DATA(p) => p.bytes() * 2,
            Self::UNCOMPRESSED_DATA(p) => p.bytes() * 2,
            Self::ZSCALE(v) => v.bytes(),
            Self::ZZERO(v) => v.bytes(),
            Self::ZBLANK(p) => p.bytes(),
            Self::Other(r) => r.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    columns: Vec<(Column, usize)>,
    bytes_per_row: usize,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            bytes_per_row: 0,
        }
    }
    pub fn push(&mut self, column: Column) {
        let bytes = column.bytes();
        self.columns.push((column, self.bytes_per_row.clone()));
        self.bytes_per_row += bytes;
    }
    pub fn bytes_per_row(&self) -> usize {
        self.bytes_per_row
    }
    pub fn columns(&self) -> &[(Column, usize)] {
        self.columns.as_ref()
    }
}
