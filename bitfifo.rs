/* A fifo... of bits! */

#[link(name = "bitfifo", vers = "0.1", author = "nejucomo@gmail.com")];
#[crate_type = "lib"];
extern mod std;
extern mod extra;

use std::uint;
use extra::ringbuf::RingBuf;
use extra::container::Deque;


struct BitFifo {
    queue: RingBuf<uint>,
    incoming: BitBucket,
    outgoing: BitBucket
}

impl BitFifo {
    fn new() -> BitFifo {
        BitFifo {
            queue: RingBuf::new(),
            incoming: BitBucket::new(),
            outgoing: BitBucket::new()
        }
    }

    fn count(&self) -> uint {
        self.incoming.count + self.outgoing.count + uint::bits * self.queue.len()
    }

    // Polymorphic push/pop:
    fn push<T: BitFifoItem>(&mut self, source: &T) {
        self.push_bits(source, source.bit_capacity());
    }

    fn push_bits<T: BitFifoItem>(&mut self, source: &T, count: uint) {
        source.push_into(self, count);
    }

    fn pop<T: BitFifoItem>(&mut self) -> T {
        self.pop_bits(full_bit_capacity::<T>())
    }

    fn pop_bits<T: BitFifoItem>(&mut self, count: uint) -> T {
        BitFifoItem::pop_from(self, count)
    }

    // Concrete BitBucket push/pop:
    fn push_bitbucket(&mut self, source: &BitBucket) {
        let total = self.incoming.count + source.count;
        assert!(total <= 2 * uint::bits);

        if total > uint::bits {
            let mut incoming = source.clone();

            let mut overflow = BitBucket::new();
            overflow.shift_in(&self.incoming);
            overflow.shift_in(&incoming.shift_out(safe_sub(uint::bits, self.incoming.count)));
            assert_eq!(overflow.count, uint::bits);
            self.queue.push_back(overflow.bits);

            self.incoming = incoming;

        } else {
            self.incoming.shift_in(source);
        }
    }

    fn pop_bitbucket(&mut self, count: uint) -> BitBucket {
        assert!(count <= uint::bits);
        assert!(count <= self.count());

        if count > self.outgoing.count {
            let mut result = self.outgoing.clone();

            match self.queue.pop_front() {
              None => {
                self.outgoing = self.incoming.clone();
                self.incoming = BitBucket::new();
              }
              Some(bits) => {
                self.outgoing = BitBucket { bits: bits, count: uint::bits }
              }
            }

            assert!(count <= self.outgoing.count + result.count);
            assert!(count >= result.count);
            result.shift_in(&self.outgoing.shift_out(safe_sub(count, result.count)));

            result

        } else {
            self.outgoing.shift_out(count)
        }
    }
}

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
        assert!(count <= self.count);
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


#[deriving(Clone)]
#[deriving(Eq)]
pub struct BitBucket {
    bits: uint,
    count: uint
}

impl BitBucket {
    fn empty() -> &'static BitBucket {
        static x: BitBucket = BitBucket { bits: 0, count: 0 };
        &x
    }

    fn new() -> BitBucket {
        BitBucket { bits: 0, count: 0 }
    }

    fn shift_in(&mut self, source: &BitBucket) {
        let total = self.count + source.count;
        assert!(total <= uint::bits);
        self.bits = (self.bits << source.count) | source.bits;
        self.count = total;
    }

    fn shift_out(&mut self, count: uint) -> BitBucket {
        if (count == 0u) {
            return BitBucket::new();
        }

        assert!(count <= self.count);

        let keep = safe_sub(self.count, count);
        let result = BitBucket {
            bits: self.bits >> keep,
            count: count
        };

        self.bits = self.bits & safe_sub(1 << keep, 1);
        self.count = keep;

        result
    }
}

fn safe_sub(a: uint, b: uint) -> uint {
    assert!(a >= b);
    a - b
}

#[cfg(test)]
mod tests {
    mod BitFifo {
        use self::utils::*;

        // BitBucket push/pop tests:
        #[test] fn fill_drain_bb_nibbles () { fill_drain_bb (bb_nibbles ()) }
        #[test] fn fill_drain_bb_bytes   () { fill_drain_bb (bb_bytes   ()) }
        #[test] fn fill_drain_bb_words   () { fill_drain_bb (bb_words   ()) }
        #[test] fn lockstep_bb_nibbles   () { lockstep_bb   (bb_nibbles ()) }
        #[test] fn lockstep_bb_bytes     () { lockstep_bb   (bb_bytes   ()) }
        #[test] fn lockstep_bb_words     () { lockstep_bb   (bb_words   ()) }

        // item push/pop tests:
        #[test] fn fill_drain_nibble_items () { fill_drain_items (bb_nibbles ()) }
        #[test] fn fill_drain_byte_items   () { fill_drain_items (bb_bytes   ()) }
        #[test] fn fill_drain_word_items   () { fill_drain_items (bb_words   ()) }
        #[test] fn lockstep_nibble_items   () { lockstep_items   (bb_nibbles ()) }
        #[test] fn lockstep_byte_items     () { lockstep_items   (bb_bytes   ()) }
        #[test] fn lockstep_word_items     () { lockstep_items   (bb_words   ()) }


        mod utils {
            use std::uint;
            use BitFifo;
            use BitFifoItem;
            use BitBucket;

            // datasets:
            pub fn bb_nibbles() -> ~[BitBucket] {
                let mut v = ~[];
                for nib in range(0u, 2^4) {
                    v.push(BitBucket { bits: nib, count: 4 });
                }
                v
            }

            pub fn bb_bytes() -> ~[BitBucket] {
                let mut v = ~[];
                for byte in range(0u, 2^8) {
                    v.push(BitBucket { bits: byte, count: 8 });
                }
                v
            }

            pub fn bb_words() -> ~[BitBucket] {
                let mut v = ~[];
                let mut word = 0x0123456789abcdef; // BUG: assumes 64 bit uint.

                for i in range(0u, 2^16) {
                    v.push(BitBucket { bits: word, count: uint::bits });
                    word = (word << 1) + i;
                }
                v
            }

            // Test implementations, given a dataset:
            pub fn fill_drain_bb(bs: &[BitBucket]) {
                test_fill_drain(bs, push_bb, pop_bb)
            }

            pub fn lockstep_bb(bs: &[BitBucket]) {
                test_lockstep(bs, push_bb, pop_bb)
            }

            pub fn fill_drain_items<T: BitFifoItem>(xs: &[T]) {
                test_fill_drain(xs, push_item, pop_item)
            }

            pub fn lockstep_items<T: BitFifoItem>(xs: &[T]) {
                test_lockstep(xs, push_item, pop_item)
            }

            // Private:
            fn push_bb(fifo: &mut BitFifo, b: &BitBucket) -> uint {
                fifo.push_bitbucket(b);
                b.count
            }

            fn pop_bb(fifo: &mut BitFifo, b: &BitBucket) -> (BitBucket, uint) {
                let out = fifo.pop_bitbucket(b.count);
                (out, b.count)
            }

            fn push_item<T: BitFifoItem>(fifo: &mut BitFifo, x: &T) -> uint {
                fifo.push(x);
                x.bit_capacity()
            }

            fn pop_item<T: BitFifoItem>(fifo: &mut BitFifo, x: &T) -> (T, uint) {
                let out = fifo.pop();
                (out, x.bit_capacity())
            }

            fn test_fill_drain<T: Eq>(xs: &[T],
                                      push: &fn(&mut BitFifo, &T) -> uint,
                                      pop: &fn(&mut BitFifo, &T) -> (T, uint))
            {
                let mut fifo = BitFifo::new();
                let mut count = 0;

                // Fill:
                for x in xs.iter() {
                    count += push(&mut fifo, x);
                    assert_eq!(fifo.count(), count);
                }

                // Drain:
                for x in xs.iter() {
                    let (out, c) = pop(&mut fifo, x);
                    assert_eq!(&out, x);
                    count -= c;
                    assert_eq!(fifo.count(), count);
                }
                assert_eq!(fifo.count(), 0);
            }

            fn test_lockstep<T: Eq>(xs: &[T],
                                    push: &fn(&mut BitFifo, &T) -> uint,
                                    pop: &fn(&mut BitFifo, &T) -> (T, uint))
            {
                let mut fifo = BitFifo::new();

                // Fill/drain in lockstep:
                for x in xs.iter() {
                    assert_eq!(fifo.count(), 0);
                    let c = push(&mut fifo, x);
                    assert_eq!(fifo.count(), c);
                    let (out, _) = pop(&mut fifo, x);
                    assert_eq!(&out, x);
                    assert_eq!(fifo.count(), 0);
                }
            }
        }
    }

    mod BitBucket {
        use self::utils::*;

        #[test]
        fn shift_out_0() {
            use std::uint;
            use BitBucket;

            let src = BitBucket { bits: 0x0123456789abcdef, count: uint::bits };
            let mut bb = src.clone();
            assert_eq!(*BitBucket::empty(), bb.shift_out(0));
            assert_eq!(src, bb);
        }

        #[test] fn all_in_all_out() { iotest(shift_in_all, shift_out_all) }
        #[test] fn all_in_chunked_out() { iotest(shift_in_all, shift_out_chunked) }
        #[test] fn chunked_in_all_out() { iotest(shift_in_chunked, shift_out_all) }
        #[test] fn chunked_in_chunked_out() { iotest(shift_in_chunked, shift_out_chunked) }


        mod utils {
            use BitBucket;

            static full: BitBucket = BitBucket { bits: 0x1b, count: 5 };
            static chunks: [BitBucket, .. 3] = [
                BitBucket { bits: 0x3, count: 2 },
                BitBucket { bits: 0x1, count: 2 },
                BitBucket { bits: 0x1, count: 1 },
                ];

            pub fn iotest(inop: &fn(&mut BitBucket), outop: &fn(&mut BitBucket)) {
                let bb = &mut BitBucket::new();
                inop(bb);
                outop(bb);
            }

            pub fn shift_in_all(dest: &mut BitBucket) {
                dest.shift_in(&full);
                assert_eq!(*dest, full);
            }

            pub fn shift_in_chunked(dest: &mut BitBucket) {
                for c in chunks.iter() {
                    dest.shift_in(c);
                }
                assert_eq!(*dest, full);
            }

            pub fn shift_out_all(src: &mut BitBucket) {
                let out = src.shift_out(full.count);
                assert_eq!(out, full);
                assert_eq!(*src, *BitBucket::empty());
            }

            pub fn shift_out_chunked(src: &mut BitBucket) {
                for c in chunks.iter() {
                    let out = src.shift_out(c.count);
                    assert_eq!(*c, out);
                }
            }
        }
    }
}

