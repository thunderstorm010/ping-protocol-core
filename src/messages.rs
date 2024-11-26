/// It is possible to create ad-hoc messages by leveraging bytemuck and the provided `LowEndianInteger` trait below.
/// Just cast the payload to your bytemuck message struct.

use bytemuck::AnyBitPattern;
use cfg_if::cfg_if;

/// Utility for using low endian integers with bytemuck
#[repr(transparent)]
#[derive(Clone, Copy, AnyBitPattern)]
pub struct LowEndianInteger<T: num_traits::PrimInt + bytemuck::AnyBitPattern> {
    t: T
}

impl<T: num_traits::PrimInt + bytemuck::AnyBitPattern> LowEndianInteger<T> {
    pub fn as_native_integer(&self) -> T {
        cfg_if! {
            if #[cfg(target_endian = "big")] {
                self.t.to_be()
            } else if #[cfg(target_endian = "little")] {
                self.t
            }
        }
    }
}
