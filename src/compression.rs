use crate::read_big_endian::ReadBigEndian;
use crate::unshuffle::Unshuffle;
use flate2::bufread::GzDecoder;
use ndarray::ArrayViewMutD;
use std::convert::AsMut;
use std::io::Read;
use std::marker::PhantomData;

pub trait Compression {
    type Original;

    fn decompress(&self, bytes: &[u8], decompressed: ArrayViewMutD<Self::Original>);
}

pub struct GZip1<T> {
    m: PhantomData<T>,
}

impl<T> GZip1<T> {
    pub fn new() -> Self {
        Self { m: PhantomData {} }
    }
}

impl<T> Compression for GZip1<T>
where
    T: ReadBigEndian,
{
    type Original = T;

    fn decompress(&self, bytes: &[u8], decompressed: ArrayViewMutD<Self::Original>) {
        let mut gz = GzDecoder::new(bytes);
        for out_pixel in decompressed {
            *out_pixel = T::read_big_endian(&mut gz);
        }
    }
}

pub struct GZip2<T> {
    m: PhantomData<T>,
}

impl<T> GZip2<T> {
    pub fn new() -> Self {
        Self { m: PhantomData {} }
    }
}

impl<T> Compression for GZip2<T>
where
    T: Unshuffle,
{
    type Original = T;

    fn decompress(&self, bytes: &[u8], decompressed: ArrayViewMutD<Self::Original>) {
        let mut gz = GzDecoder::new(bytes);
        let n_pixels = decompressed.len();
        let n_bytes = n_pixels * std::mem::size_of::<T>();
        let mut unzipped = Vec::with_capacity(n_bytes);
        gz.read_to_end(&mut unzipped).unwrap();
        T::unshuffle(&unzipped, decompressed)
    }
}

pub struct NoCompress<T> {
    m: PhantomData<T>,
}

impl<T> NoCompress<T> {
    pub fn new() -> Self {
        Self { m: PhantomData {} }
    }
}

impl<T> Compression for NoCompress<T>
where
    T: ReadBigEndian,
{
    type Original = T;

    fn decompress(&self, bytes: &[u8], decompressed: ArrayViewMutD<Self::Original>) {
        // This copy wouldn't be necessary if we had a version of ReadBigEndian
        // that could work on an iterator instead of std::io::Read.
        let mut v = Vec::from(bytes);
        let mut bytes: &[u8] = v.as_mut();
        for out_pixel in decompressed {
            *out_pixel = T::read_big_endian(&mut bytes);
        }
    }
}
