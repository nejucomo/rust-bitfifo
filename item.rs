use std::{uint, u64, u32, u16, u8};


use BitFifo;
use bitbucket::BitBucket;


pub trait Item : Eq {
    fn push_into(&self, fifo: &mut BitFifo, count: uint);

    fn pop_from(fifo: &mut BitFifo, count: uint) -> Self;

    fn bit_capacity(&self) -> uint { full_bit_capacity::<Self>() }

    /* This is a workaround pattern taked from libstd/num/num.rs.
     * See rust ticket #8888; callers can use this convenience function:

         full_bit_capacity::<T>()
     */
    fn _full_bit_capacity(unused_self: Option<Self>) -> uint;
}

pub fn full_bit_capacity<T: Item>() -> uint {
    let x: Option<T> = None;
    Item::_full_bit_capacity(x)
}


// Implementations:
impl Item for BitBucket {
    fn push_into(&self, fifo: &mut BitFifo, count: uint) {
        assert_le!(count, self.count);
        if (count < self.count) {
            fifo.push_bitbucket(&BitBucket { bits: self.bits, count: count });
        } else {
            fifo.push_bitbucket(self);
        }
    }

    fn pop_from(fifo: &mut BitFifo, count: uint) -> BitBucket { fifo.pop_bitbucket(count) }

    fn bit_capacity(&self) -> uint { self.count }

    fn _full_bit_capacity(_: Option<BitBucket>) -> uint { uint::bits }
}


macro_rules! ui_impl (
    ($T:ty, $bits:expr) => (
        impl Item for $T {
            fn push_into(&self, fifo: &mut BitFifo, count: uint) {
                assert_le!(count, $bits);
                fifo.push_bitbucket(&BitBucket { bits: *self as uint, count: count });
            }

            fn pop_from(fifo: &mut BitFifo, count: uint) -> $T {
                let bucket = fifo.pop_bitbucket(count);
                bucket.bits as $T
            }

            fn bit_capacity(&self) -> uint { $bits }

            fn _full_bit_capacity(_: Option<$T>) -> uint { $bits }
        }
    )
)


ui_impl!(uint, uint::bits)
ui_impl!(u64, u64::bits)
ui_impl!(u32, u32::bits)
ui_impl!(u16, u16::bits)
ui_impl!(u8, u8::bits)
