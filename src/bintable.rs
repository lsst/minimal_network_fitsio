use std::convert::Into;

pub trait ReadColumn {
    type Value;
    fn bytes(&self) -> usize;
    fn read(&self, row: &[u8], offset: usize) -> Self::Value;
}

pub struct Address {
    pub size: usize,
    pub offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressType {
    I32,
    I64,
}

impl ReadColumn for AddressType {
    type Value = Option<Address>;
    fn bytes(&self) -> usize {
        match self {
            Self::I32 => 8,
            Self::I64 => 16,
        }
    }
    fn read(&self, row: &[u8], offset: usize) -> Self::Value {
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

impl Into<char> for AddressType {
    fn into(self) -> char {
        match self {
            Self::I32 => 'P',
            Self::I64 => 'Q',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerType {
    U8,
    I16,
    I32,
    I64,
}

impl ReadColumn for IntegerType {
    type Value = i64;
    fn bytes(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
        }
    }
    fn read(&self, row: &[u8], offset: usize) -> Self::Value {
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

impl ReadColumn for FloatType {
    type Value = f64;
    fn bytes(&self) -> usize {
        match self {
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }
    fn read(&self, row: &[u8], offset: usize) -> Self::Value {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OffsetColumn<T> {
    type_: T,
    offset: usize,
}

impl<T: ReadColumn> OffsetColumn<T> {
    pub fn new(type_: T, offset: &mut usize) -> Self {
        let result = Self {
            type_,
            offset: *offset,
        };
        *offset += result.type_.bytes();
        result
    }
    pub fn bytes(&self) -> usize {
        self.type_.bytes()
    }
    pub fn read(&self, row: &[u8]) -> T::Value {
        self.type_.read(row, self.offset)
    }
}
