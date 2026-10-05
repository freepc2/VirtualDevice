/// Scalar values that convert to and from little-endian bytes.
pub trait AnalogValue: Copy + Sized {
    const SIZE: usize;

    fn write_le(self, dst: &mut [u8]);
    fn read_le(src: &[u8]) -> Self;
}

macro_rules! impl_analog_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AnalogValue for $ty {
                const SIZE: usize = core::mem::size_of::<$ty>();

                #[inline]
                fn write_le(self, dst: &mut [u8]) {
                    dst.copy_from_slice(&self.to_le_bytes());
                }

                #[inline]
                fn read_le(src: &[u8]) -> Self {
                    let mut bytes = [0u8; core::mem::size_of::<$ty>()];
                    bytes.copy_from_slice(src);
                    <$ty>::from_le_bytes(bytes)
                }
            }
        )+
    };
}

impl_analog_value!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
