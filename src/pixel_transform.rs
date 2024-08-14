use crate::lossy_from::LossyFrom;
use std::ops::{Mul, Sub};

pub trait PixelTransform {
    type Original;
    type Stored;

    fn reverse(&mut self, s: Self::Stored) -> Self::Original;
}

#[derive(Clone)]
pub struct I8Transform();

impl PixelTransform for I8Transform {
    type Original = i8;
    type Stored = u8;

    fn reverse(&mut self, s: Self::Stored) -> Self::Original {
        (-128i8).wrapping_add_unsigned(s)
    }
}

macro_rules! unsigned_transform {
    ($name: ident, $zero: expr, $original: ty, $stored: ty) => {
        #[derive(Clone)]
        pub struct $name();
        impl PixelTransform for $name {
            type Original = $original;
            type Stored = $stored;

            fn reverse(&mut self, s: Self::Stored) -> Self::Original {
                ($zero).wrapping_add_signed(s)
            }
        }
    };
}

unsigned_transform!(U16Transform, 32768_u16, u16, i16);
unsigned_transform!(U32Transform, 2147483648_u32, u32, i32);
unsigned_transform!(U64Transform, 9223372036854775808_u64, u64, i64);

pub trait NaN {
    fn nan() -> Self;
}

impl NaN for f32 {
    fn nan() -> Self {
        f32::NAN
    }
}

impl NaN for f64 {
    fn nan() -> Self {
        f64::NAN
    }
}

#[derive(Clone)]
pub struct NoDitherQuantization<F, I> {
    zero: F,
    scale: F,
    blank: Option<I>,
}

impl<F, I> NoDitherQuantization<F, I> {
    pub fn new(zero: F, scale: F, blank: Option<I>) -> Self {
        Self { zero, scale, blank }
    }
}

impl<I, F> PixelTransform for NoDitherQuantization<F, I>
where
    F: Copy,
    I: Copy,
    I: Eq,
    F: LossyFrom<I>,
    F: NaN,
    F: Mul<F, Output = F>,
    F: Sub<F, Output = F>,
{
    type Original = F;
    type Stored = I;

    fn reverse(&mut self, s: Self::Stored) -> Self::Original {
        if let Some(blank) = self.blank {
            if s == blank {
                return F::nan();
            }
        }
        // Might be worth using mul_add here in the future, but there's no
        // stdlib trait for that.
        F::lossy_from(s) * self.scale - self.zero
    }
}
