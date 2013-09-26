use std::{uint, u64, u32, u16, u8};


use BitFifo;
use bitbucket::BitBucket;


pub trait Item : Eq {
    /* Push this item into fifo.  If count is None, push all bits in
     * this item, else it should be Some(c) where c is less than or equal
     * to the number of bits in this item.
     */
    fn push_into(&self, fifo: &mut BitFifo, count: Option<uint>);

    /* Pop an item from a fifo; if count is None pop as many bits as
     * can fit in an item of this type.  If count is Some(c), then the
     * caller must ensure c is less than full capacity.
     */
    fn pop_from(fifo: &mut BitFifo, count: Option<uint>) -> Self;

    // How many bits are in this Item?
    fn bit_count(&self) -> uint { 0 }
}


// Implementations:
impl Item for BitBucket {
    fn push_into(&self, fifo: &mut BitFifo, count: Option<uint>) {
        let c = match count { None => self.count, Some(c) => c };

        assert_le!(c, self.count);

        if (c < self.count) {
            fifo.push_bitbucket(&BitBucket { bits: self.bits, count: c });
        } else {
            fifo.push_bitbucket(self);
        }
    }

    fn pop_from(fifo: &mut BitFifo, count: Option<uint>) -> BitBucket {
        let c = match count { None => uint::bits, Some(c) => c };
        fifo.pop_bitbucket(c)
    }

    fn bit_count(&self) -> uint { self.count }
}


macro_rules! ui_impl (
    ($T:ty, $bits:expr) => (
        impl Item for $T {
            fn push_into(&self, fifo: &mut BitFifo, count: Option<uint>) {
                let c = match count { None => $bits, Some(c) => c };
                assert_le!(c, $bits);
                fifo.push_bitbucket(&BitBucket { bits: *self as uint, count: c });
            }

            fn pop_from(fifo: &mut BitFifo, count: Option<uint>) -> $T {
                let bucket: BitBucket = fifo.pop(count);
                bucket.bits as $T
            }

            fn bit_count(&self) -> uint { $bits }
        }
    )
)


ui_impl!(uint, uint::bits)
ui_impl!(u64, u64::bits)
ui_impl!(u32, u32::bits)
ui_impl!(u16, u16::bits)
ui_impl!(u8, u8::bits)
