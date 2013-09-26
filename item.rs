use std::{uint, u64, u32, u16, u8};


use BitCount;
use BitFifo;
use bitbucket::BitBucket;


pub trait Item : Eq {
    fn push_into(&self, fifo: &mut BitFifo, limit: Option<BitCount>);

    fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> Self;

    fn bit_count(&self) -> BitCount;

    fn bit_capacity(Option<Self>) -> Option<BitCount>;
}


pub fn item_capacity<T: Item>() -> Option<BitCount> {
    let x: Option<T> = None;
    Item::bit_capacity(x)
}


// Implementations:
impl Item for BitBucket {
    fn push_into(&self, fifo: &mut BitFifo, limit: Option<BitCount>) {
        fifo.push_bitbucket(
            &BitBucket {
                bits: self.bits,
                count: get_push_limit(self, limit)
            });
    }

    fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> BitBucket {
        let c = get_pop_limit::<BitBucket>(fifo, limit);
        fifo.pop_bitbucket(c)
    }

    fn bit_count(&self) -> BitCount { self.count }

    fn bit_capacity(_: Option<BitBucket>) -> Option<BitCount> { Some(uint::bits) }
}


macro_rules! ui_impl (
    ($T:ty, $bits:expr) => (
        impl Item for $T {
            fn push_into(&self, fifo: &mut BitFifo, limit: Option<BitCount>) {
                fifo.push_bitbucket(
                    &BitBucket {
                        bits: *self as uint,
                        count: get_push_limit(self, limit)
                    });
            }

            fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> $T {
                let bucket: BitBucket = fifo.pop(limit);
                bucket.bits as $T
            }

            fn bit_count(&self) -> BitCount { $bits }

            fn bit_capacity(_: Option<$T>) -> Option<BitCount> { Some($bits) }
        }
    )
)


ui_impl!(uint, uint::bits)
ui_impl!(u64, u64::bits)
ui_impl!(u32, u32::bits)
ui_impl!(u16, u16::bits)
ui_impl!(u8, u8::bits)


// Vectors:
impl<T: Item> Item for ~[T] {
    fn push_into(&self, fifo: &mut BitFifo, limit: Option<BitCount>) {
        let mut remaining = limit;

        let limit_reached = || {
            match remaining {
                None => false,
                Some(c) => c == 0
            }
        };

        for x in self.iter() {
            if limit_reached() {
                break;
            }

            let sublimit = get_push_limit(x, remaining);
            x.push_into(fifo, Some(sublimit));

            remaining = remaining.map( |l| l - sublimit );
        }
    }

    fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> ~[T] {
        let mut remaining = limit;

        let limit_reached = || {
            match remaining {
                None => false,
                Some(c) => c == 0
            }
        };

        let mut result = ~[];

        while fifo.count() > 0 && !limit_reached() {
            let sublimit = get_pop_limit::<T>(fifo, remaining);
            result.push(Item::pop_from(fifo, Some(sublimit)));
            remaining = remaining.map( |l| l - sublimit );
        }

        result
    }

    fn bit_count(&self) -> BitCount {
        self.iter().fold( 0, |sum, x| sum + x.bit_count() )
    }

    fn bit_capacity(_: Option<~[T]>) -> Option<BitCount> {
        None
    }
}


// Internal utilities:
fn get_push_limit<T: Item>(item: &T, limit: Option<BitCount>) -> BitCount {
    opt_min(item.bit_count(), limit)
}

fn get_pop_limit<T: Item>(fifo: &BitFifo, limit: Option<BitCount>) -> BitCount {
    opt_min(opt_min(fifo.count(), item_capacity::<T>()), limit)
}

fn opt_min(a: uint, optb: Option<uint>) -> uint {
    optb.map_default(a, |b| a.min(b))
}

