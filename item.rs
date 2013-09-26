use std::{uint, u64, u32, u16, u8};


use BitFifo;
use bitbucket::BitBucket;


pub trait Item : Eq {
    fn push_into(&self, fifo: &mut BitFifo, limit: Option<uint>);

    fn pop_from(fifo: &mut BitFifo, limit: Option<uint>) -> Self;

    fn bit_count(&self) -> uint;

    fn bit_capacity(Option<Self>) -> Option<uint>;
}


pub fn item_capacity<T: Item>() -> Option<uint> {
    let x: Option<T> = None;
    Item::bit_capacity(x)
}


// Implementations:
impl Item for BitBucket {
    fn push_into(&self, fifo: &mut BitFifo, limit: Option<uint>) {
        fifo.push_bitbucket(
            &BitBucket {
                bits: self.bits,
                count: get_push_limit(self, limit)
            });
    }

    fn pop_from(fifo: &mut BitFifo, limit: Option<uint>) -> BitBucket {
        let c = get_pop_limit::<BitBucket>(fifo, limit);
        fifo.pop_bitbucket(c)
    }

    fn bit_count(&self) -> uint { self.count }

    fn bit_capacity(_: Option<BitBucket>) -> Option<uint> { Some(uint::bits) }
}


macro_rules! ui_impl (
    ($T:ty, $bits:expr) => (
        impl Item for $T {
            fn push_into(&self, fifo: &mut BitFifo, limit: Option<uint>) {
                fifo.push_bitbucket(
                    &BitBucket {
                        bits: *self as uint,
                        count: get_push_limit(self, limit)
                    });
            }

            fn pop_from(fifo: &mut BitFifo, limit: Option<uint>) -> $T {
                let bucket: BitBucket = fifo.pop(limit);
                bucket.bits as $T
            }

            fn bit_count(&self) -> uint { $bits }

            fn bit_capacity(_: Option<$T>) -> Option<uint> { Some($bits) }
        }
    )
)


ui_impl!(uint, uint::bits)
ui_impl!(u64, u64::bits)
ui_impl!(u32, u32::bits)
ui_impl!(u16, u16::bits)
ui_impl!(u8, u8::bits)


// Internal utilities:
fn get_push_limit<T: Item>(item: &T, limit: Option<uint>) -> uint {
    opt_min(item.bit_count(), limit)
}

fn get_pop_limit<T: Item>(fifo: &BitFifo, limit: Option<uint>) -> uint {
    opt_min(opt_min(fifo.count(), item_capacity::<T>()), limit)
}

fn opt_min(a: uint, optb: Option<uint>) -> uint {
    optb.map_default(a, |b| a.min(b))
}

