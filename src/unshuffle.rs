use ndarray::ArrayViewMutD;
use std::iter::Iterator;

struct Unshuffle2<'a> {
    i1: std::slice::Iter<'a, u8>,
    i2: std::slice::Iter<'a, u8>,
}

impl<'a> Unshuffle2<'a> {
    fn new(data: &'a [u8]) -> Self {
        assert!(data.len() % 2 == 0);
        let n = data.len() / 2;
        let (s1, s2) = data.split_at(n);
        Self {
            i1: s1.iter(),
            i2: s2.iter(),
        }
    }
}

impl<'a> Iterator for Unshuffle2<'a> {
    type Item = [u8; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let b1 = self.i1.next()?;
        let b2 = self.i2.next().unwrap();
        Some([*b1, *b2])
    }
}

struct Unshuffle4<'a> {
    i1: std::slice::Iter<'a, u8>,
    i2: std::slice::Iter<'a, u8>,
    i3: std::slice::Iter<'a, u8>,
    i4: std::slice::Iter<'a, u8>,
}

impl<'a> Unshuffle4<'a> {
    fn new(data: &'a [u8]) -> Self {
        assert!(data.len() % 4 == 0);
        let n = data.len() / 4;
        let (s1, data) = data.split_at(n);
        let (s2, data) = data.split_at(n);
        let (s3, s4) = data.split_at(n);
        Self {
            i1: s1.iter(),
            i2: s2.iter(),
            i3: s3.iter(),
            i4: s4.iter(),
        }
    }
}

impl<'a> Iterator for Unshuffle4<'a> {
    type Item = [u8; 4];

    fn next(&mut self) -> Option<Self::Item> {
        let b1 = self.i1.next()?;
        let b2 = self.i2.next().unwrap();
        let b3 = self.i3.next().unwrap();
        let b4 = self.i4.next().unwrap();
        Some([*b1, *b2, *b3, *b4])
    }
}

struct Unshuffle8<'a> {
    i1: std::slice::Iter<'a, u8>,
    i2: std::slice::Iter<'a, u8>,
    i3: std::slice::Iter<'a, u8>,
    i4: std::slice::Iter<'a, u8>,
    i5: std::slice::Iter<'a, u8>,
    i6: std::slice::Iter<'a, u8>,
    i7: std::slice::Iter<'a, u8>,
    i8: std::slice::Iter<'a, u8>,
}

impl<'a> Unshuffle8<'a> {
    fn new(data: &'a [u8]) -> Self {
        assert!(data.len() % 8 == 0);
        let n = data.len() / 8;
        let (s1, data) = data.split_at(n);
        let (s2, data) = data.split_at(n);
        let (s3, data) = data.split_at(n);
        let (s4, data) = data.split_at(n);
        let (s5, data) = data.split_at(n);
        let (s6, data) = data.split_at(n);
        let (s7, s8) = data.split_at(n);
        Self {
            i1: s1.iter(),
            i2: s2.iter(),
            i3: s3.iter(),
            i4: s4.iter(),
            i5: s5.iter(),
            i6: s6.iter(),
            i7: s7.iter(),
            i8: s8.iter(),
        }
    }
}

impl<'a> Iterator for Unshuffle8<'a> {
    type Item = [u8; 8];

    fn next(&mut self) -> Option<Self::Item> {
        let b1 = self.i1.next()?;
        let b2 = self.i2.next().unwrap();
        let b3 = self.i3.next().unwrap();
        let b4 = self.i4.next().unwrap();
        let b5 = self.i5.next().unwrap();
        let b6 = self.i6.next().unwrap();
        let b7 = self.i7.next().unwrap();
        let b8 = self.i8.next().unwrap();
        Some([*b1, *b2, *b3, *b4, *b5, *b6, *b7, *b8])
    }
}

pub trait Unshuffle: Copy {
    fn unshuffle(data: &[u8], out: ArrayViewMutD<Self>);
}

impl Unshuffle for u8 {
    fn unshuffle(data: &[u8], mut out: ArrayViewMutD<u8>) {
        out.iter_mut().zip(data.iter()).for_each(|(o, i)| *o = *i)
    }
}

macro_rules! unshuffle_impl {
    ($t:ty, $iter:ty) => {
        impl Unshuffle for $t {
            fn unshuffle(data: &[u8], mut out: ArrayViewMutD<$t>) {
                out.iter_mut()
                    .zip(<$iter>::new(data))
                    .for_each(|(o, i)| *o = <$t>::from_be_bytes(i))
            }
        }
    };
}

unshuffle_impl!(i16, Unshuffle2);
unshuffle_impl!(i32, Unshuffle4);
unshuffle_impl!(f32, Unshuffle4);
unshuffle_impl!(i64, Unshuffle8);
unshuffle_impl!(f64, Unshuffle8);
