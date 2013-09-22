use std::uint;


use BitFifo;
use bitbucket::BitBucket;


pub trait BitFifoItem : Eq {
    fn push_into(&self, fifo: &mut BitFifo, count: uint);

    fn pop_from(fifo: &mut BitFifo, count: uint) -> Self;

    fn bit_capacity(&self) -> uint { full_bit_capacity::<Self>() }

    /* This is a workaround pattern taked from libstd/num/num.rs.
     * See rust ticket #8888; callers can use this convenience function:

         full_bit_capacity::<T>()
     */
    fn _full_bit_capacity(unused_self: Option<Self>) -> uint;
}

pub fn full_bit_capacity<T: BitFifoItem>() -> uint {
    let x: Option<T> = None;
    BitFifoItem::_full_bit_capacity(x)
}

impl BitFifoItem for BitBucket {
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
