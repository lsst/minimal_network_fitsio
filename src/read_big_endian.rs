pub trait ReadBigEndian {
    fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self;
}

impl ReadBigEndian for u8 {
    fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self {
        let mut bytes: [u8; 1] = [0; 1];
        reader.read_exact(&mut bytes).unwrap();
        bytes[0]
    }
}

impl ReadBigEndian for i16 {
    fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self {
        let mut bytes: [u8; 2] = [0; 2];
        reader.read_exact(&mut bytes).unwrap();
        i16::from_be_bytes(bytes)
    }
}

impl ReadBigEndian for i32 {
    fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self {
        let mut bytes: [u8; 4] = [0; 4];
        reader.read_exact(&mut bytes).unwrap();
        i32::from_be_bytes(bytes)
    }
}

impl ReadBigEndian for i64 {
    fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self {
        let mut bytes: [u8; 8] = [0; 8];
        reader.read_exact(&mut bytes).unwrap();
        i64::from_be_bytes(bytes)
    }
}

impl ReadBigEndian for f32 {
    fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self {
        let mut bytes: [u8; 4] = [0; 4];
        reader.read_exact(&mut bytes).unwrap();
        f32::from_be_bytes(bytes)
    }
}

impl ReadBigEndian for f64 {
    fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self {
        let mut bytes: [u8; 8] = [0; 8];
        reader.read_exact(&mut bytes).unwrap();
        f64::from_be_bytes(bytes)
    }
}
