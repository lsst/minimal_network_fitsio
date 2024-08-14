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

macro_rules! read_big_endian_impl {
    ($t: ty, $n: expr) => {
        impl ReadBigEndian for $t {
            fn read_big_endian<R: std::io::Read>(reader: &mut R) -> Self {
                let mut bytes: [u8; $n] = [0; $n];
                reader.read_exact(&mut bytes).unwrap();
                <$t>::from_be_bytes(bytes)
            }
        }
    };
}

read_big_endian_impl!(i16, 2);
read_big_endian_impl!(i32, 4);
read_big_endian_impl!(i64, 8);
read_big_endian_impl!(f32, 4);
read_big_endian_impl!(f64, 8);
