use crate::bintable::Address;
use crate::pixel_transform::PixelTransform;
use ndarray::{ArrayD, ArrayViewMutD};

use crate::compression::Compression;

pub trait TileReader<P> {
    fn address(&self) -> &Address;
    fn shape(&self) -> &[usize];
    fn read(&self, data: &[u8], out: ArrayViewMutD<P>);
}

pub enum TypedTileReader {
    U8(Box<dyn TileReader<u8>>),
    I8(Box<dyn TileReader<i8>>),
    U16(Box<dyn TileReader<u16>>),
    I16(Box<dyn TileReader<i16>>),
    U32(Box<dyn TileReader<u32>>),
    I32(Box<dyn TileReader<i32>>),
    U64(Box<dyn TileReader<u64>>),
    I64(Box<dyn TileReader<i64>>),
    F32(Box<dyn TileReader<f32>>),
    F64(Box<dyn TileReader<f64>>),
}

pub struct BasicTileReader<C> {
    address: Address,
    shape: Vec<usize>,
    compression: C,
}

impl<C: Compression + 'static> BasicTileReader<C> {
    pub fn new(address: Address, shape: Vec<usize>, compression: C) -> Self {
        Self {
            address,
            shape,
            compression,
        }
    }
    pub fn new_boxed(
        address: Address,
        shape: Vec<usize>,
        compression: C,
    ) -> Box<dyn TileReader<C::Original>> {
        Box::new(Self::new(address, shape, compression))
    }
    pub fn transformed<T>(self, transform: T) -> Box<dyn TileReader<T::Original>>
    where
        T: PixelTransform<Stored = C::Original> + Clone + 'static,
        T::Stored: Clone + Default,
    {
        Box::new(TransformedTileReader {
            base: self,
            transform,
        })
    }
}

impl<C: Compression> TileReader<C::Original> for BasicTileReader<C> {
    fn address(&self) -> &Address {
        &self.address
    }
    fn shape(&self) -> &[usize] {
        &self.shape
    }
    fn read(&self, data: &[u8], out: ArrayViewMutD<C::Original>) {
        self.compression.decompress(data, out)
    }
}

pub struct TransformedTileReader<B, T> {
    base: B,
    transform: T,
}

impl<B, T> TileReader<T::Original> for TransformedTileReader<B, T>
where
    T: PixelTransform + Clone,
    B: TileReader<T::Stored>,
    T::Stored: std::default::Default + Clone,
{
    fn address(&self) -> &Address {
        self.base.address()
    }
    fn shape(&self) -> &[usize] {
        self.base.shape()
    }
    fn read(&self, data: &[u8], mut out: ArrayViewMutD<T::Original>) {
        let mut decompressed: ArrayD<T::Stored> =
            ArrayD::from_elem(out.shape(), T::Stored::default());
        self.base.read(data, decompressed.view_mut());
        let mut t = self.transform.clone();
        out.iter_mut()
            .zip(decompressed.iter())
            .for_each(|(o, i)| *o = t.reverse(i.clone()))
    }
}
