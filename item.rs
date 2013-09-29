/* Copyright 2013 - Nathan Wilcox - Distributed under the terms of the
 * TGPPLv1 or at your option any later version.  See ./COPYING.TGPPL.rst
 * for details.
 */

use std::{uint, u64, u32, u16, u8};


use BitCount;
use BitFifo;


pub trait Pushable {
    fn push_into(&self, fifo: &mut BitFifo, limit: Option<BitCount>);
    fn bit_count(&self) -> BitCount;
}

pub trait Poppable {
    fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> (Self, BitCount);
    fn bit_capacity(Option<Self>) -> Option<BitCount>;
}


pub fn poppable_capacity<T: Poppable>() -> Option<BitCount> {
    let x: Option<T> = None;
    Poppable::bit_capacity(x)
}


// Implementations:
impl Pushable for bool {
    fn push_into(&self, fifo: &mut BitFifo, limit: Option<BitCount>) {
        let bits = match *self { false => 0, true => 1 };
        fifo.push_uint(bits, get_push_limit(self, limit));
    }

    fn bit_count(&self) -> BitCount { 1u }
}

impl Poppable for bool {
    fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> (bool, BitCount) {
        let c = get_pop_limit::<bool>(fifo, limit);
        let (bits, count) = fifo.pop_uint(c);
        let result = match bits {
            0u => false,
            1u => true,
            _ => fail!("Invalid boolean bit pattern: %x", bits)
        };
        (result, count)
    }

    fn bit_capacity(_: Option<bool>) -> Option<BitCount> { Some(1u) }
}


macro_rules! uint_pushable_impl (
    ($T:ty, $bits:expr) => (
        impl Pushable for $T {
            fn push_into(&self, fifo: &mut BitFifo, limit: Option<BitCount>) {
                fifo.push_uint(*self as uint, get_push_limit(self, limit));
            }

            fn bit_count(&self) -> BitCount { $bits }
        }
    )
)

macro_rules! uint_poppable_impl (
    ($T:ty, $bits:expr) => (
        impl Poppable for $T {
            fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> ($T, BitCount) {
                let count = get_pop_limit::<$T>(fifo, limit);
                let (bits, count) = fifo.pop_uint(count);
                (bits as $T, count)
            }

            fn bit_capacity(_: Option<$T>) -> Option<BitCount> { Some($bits) }
        }
    )
)


uint_pushable_impl!(uint, uint::bits)
uint_poppable_impl!(uint, uint::bits)
uint_pushable_impl!(u64, u64::bits)
uint_poppable_impl!(u64, u64::bits)
uint_pushable_impl!(u32, u32::bits)
uint_poppable_impl!(u32, u32::bits)
uint_pushable_impl!(u16, u16::bits)
uint_poppable_impl!(u16, u16::bits)
uint_pushable_impl!(u8, u8::bits)
uint_poppable_impl!(u8, u8::bits)


// Vectors:
impl<'self, T: Pushable> Pushable for &'self [T] {
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

    fn bit_count(&self) -> BitCount {
        self.iter().fold( 0, |sum, x| sum + x.bit_count() )
    }
}


impl<T: Poppable> Poppable for ~[T] {
    fn pop_from(fifo: &mut BitFifo, limit: Option<BitCount>) -> (~[T], BitCount) {
        let mut remaining = limit;
        let mut count = 0;

        let limit_reached = || {
            match remaining {
                None => false,
                Some(c) => c == 0
            }
        };

        let mut result = ~[];

        while fifo.count() > 0 && !limit_reached() {
            let sublimit = get_pop_limit::<T>(fifo, remaining);
            let (elem, subcount) = Poppable::pop_from(fifo, Some(sublimit));
            result.push(elem);
            remaining = remaining.map( |l| l - subcount );
            count += subcount
        }

        (result, count)
    }

    fn bit_capacity(_: Option<~[T]>) -> Option<BitCount> {
        None
    }
}


// Internal utilities:
fn get_push_limit<T: Pushable>(item: &T, limit: Option<BitCount>) -> BitCount {
    opt_min(item.bit_count(), limit)
}

fn get_pop_limit<T: Poppable>(fifo: &BitFifo, limit: Option<BitCount>) -> BitCount {
    opt_min(opt_min(fifo.count(), poppable_capacity::<T>()), limit)
}

fn opt_min(a: uint, optb: Option<uint>) -> uint {
    optb.map_default(a, |b| a.min(b))
}

