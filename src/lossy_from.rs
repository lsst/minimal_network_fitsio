pub trait LossyFrom<T> {
    fn lossy_from(v: T) -> Self;
}

macro_rules! lossy_from_impl {
    ($a: ty, $b: ty) => {
        impl LossyFrom<$a> for $b {
            fn lossy_from(v: $a) -> Self {
                v as $b
            }
        }
    };
}

lossy_from_impl!(u8, f32);
lossy_from_impl!(i16, f32);
lossy_from_impl!(i32, f32);
lossy_from_impl!(i64, f32);
lossy_from_impl!(u8, f64);
lossy_from_impl!(i16, f64);
lossy_from_impl!(i32, f64);
lossy_from_impl!(i64, f64);
